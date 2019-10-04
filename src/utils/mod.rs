use std::{net::IpAddr, path::PathBuf};

pub mod cli {
    use serde_json;
    use std::io;

    #[derive(Debug, Deserialize)]
    struct Instance {
        instance_id: String,
    }

    pub fn read_instance_ids(ids: &[&str]) -> Result<Vec<String>> {
        let instance_ids: Vec<_> = ids.iter().map(|s| s.to_string()).collect();

        // Let's check if we shall read instance ids from stdin
        if instance_ids.len() == 1 && instance_ids[0] == "-" {
            read_instance_ids_from_stdin()
        } else {
            Ok(instance_ids)
        }
    }

    fn read_instance_ids_from_stdin() -> Result<Vec<String>> {
        let instances: Vec<Instance> =
            serde_json::from_reader(io::stdin()).chain_err(|| ErrorKind::FailedToReadStdin)?;

        let instance_ids: Vec<String> = instances.into_iter().map(|i| i.instance_id).collect();

        Ok(instance_ids)
    }

    error_chain! {
        errors {
            FailedToReadStdin {
                description("Failed to read instance ids from stdin")
            }
        }
    }
}

pub mod command {
    use super::*;

    use std::{fs::File, time::Duration};
    use subprocess::{Exec, ExitStatus as SubprocessExitStatus, Redirection};

    #[derive(Debug)]
    pub struct Command {
        pub id: String,
        pub cmd: String,
        pub args: Option<Vec<String>>,
        pub cwd: Option<String>,
        pub log: PathBuf,
        pub timeout: Option<Duration>,
    }

    #[derive(Debug, Serialize)]
    pub struct CommandResult {
        pub id: String,
        pub log: PathBuf,
        pub exit_status: ExitStatus,
    }

    #[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize)]
    pub enum ExitStatus {
        Exited(u32),
        Signaled(u8),
        Other(i32),
        Undetermined,
    }

    impl ExitStatus {
        pub fn success(self) -> bool {
            if let ExitStatus::Exited(0) = self {
                true
            } else {
                false
            }
        }
    }
    impl From<SubprocessExitStatus> for ExitStatus {
        fn from(s: SubprocessExitStatus) -> Self {
            match s {
                SubprocessExitStatus::Exited(x) => ExitStatus::Exited(x),
                SubprocessExitStatus::Signaled(x) => ExitStatus::Signaled(x),
                SubprocessExitStatus::Other(x) => ExitStatus::Other(x),
                SubprocessExitStatus::Undetermined => ExitStatus::Undetermined,
            }
        }
    }

    impl Command {
        pub fn run<T: Fn() -> ()>(self, progress: Option<T>) -> Result<CommandResult> {
            debug!("Executing command '{:?}'", self);
            let cmd = self.cmd.clone();
            let mut c = if let Some(ref args) = self.args {
                Exec::cmd(&cmd).args(args)
            } else {
                Exec::cmd(&cmd)
            };

            c = if let Some(cwd) = self.cwd {
                c.cwd(cwd)
            } else {
                c
            };

            let mut p = c
                .stdout(File::create(self.log.clone()).unwrap())
                .stderr(Redirection::Merge)
                .popen()
                .chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;

            let mut timeout = self.timeout;
            let resolution = Duration::from_millis(100);
            loop {
                let status = p
                    .wait_timeout(resolution)
                    .chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;

                if let Some(count_down) = timeout {
                    let count_down = count_down - resolution;
                    if count_down <= Duration::from_secs(0) {
                        p.kill()
                            .chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;
                        let exit_status = p
                            .wait()
                            .chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;
                        return Ok(CommandResult {
                            id: self.id,
                            log: self.log,
                            exit_status: exit_status.into(),
                        });
                    }
                    timeout = Some(count_down);
                }

                if let Some(ref progress) = progress {
                    progress();
                }
                if let Some(exit_status) = status {
                    return Ok(CommandResult {
                        id: self.id,
                        log: self.log,
                        exit_status: exit_status.into(),
                    });
                }
            }
        }
    }
}

pub mod run {
    use super::*;

    use clams::prelude::*;
    use std::{fs::File, sync::mpsc::channel, thread};

    use output::{
        instances::{JsonOutputCommandResults, OutputCommandResults, TableOutputCommandResults},
        OutputType,
    };
    use utils::command::{Command, CommandResult, ExitStatus};

    pub fn run(commands: Vec<Command>, use_progress_bar: bool) -> Result<Vec<CommandResult>> {
        if use_progress_bar {
            debug!("Running commands with progress bar.");
            run_with_progress(commands)
        } else {
            debug!("Running commands without progress bar.");
            run_without_progress(commands)
        }
        .chain_err(|| ErrorKind::FailedToRunCommands)
    }

    pub fn run_without_progress(commands: Vec<Command>) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();

        for cmd in commands.into_iter() {
            let (sender, receiver) = channel();
            results.push(receiver);

            let _ = thread::spawn(move || {
                let res = cmd.run(None::<fn()>);
                sender.send(res).unwrap();
            });
        }

        results
            .iter()
            .map(|r| {
                r.recv().unwrap()
                    // TODO: Error should contain the command.
                    .map_err(|e| Error::with_chain(e, ErrorKind::FailedToRunCommand("<nyi>".to_owned())))
            })
            .collect()
    }

    pub fn run_with_progress(commands: Vec<Command>) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();
        let m = MultiProgress::new();

        for cmd in commands.into_iter() {
            let (sender, receiver) = channel();
            results.push(receiver);

            let spinner_style = ProgressStyle::default_clams_spinner();
            let pb = m.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style);
            pb.set_prefix(&cmd.id.to_string());
            pb.set_message(&cmd.cmd);

            let log_path = cmd.log.clone();
            let _ = thread::spawn(move || {
                let progress = || {
                    let line = File::open(log_path.clone())
                        .unwrap()
                        .read_last_line()
                        .unwrap();

                    pb.set_message(&format!("Running: {}", line));
                    pb.inc(1);
                };

                let res = cmd.run(Some(progress));

                let finish_msg = match res {
                    Ok(CommandResult {
                        exit_status: ExitStatus::Exited(0),
                        ..
                    }) => format!("{}.", "Done".green()),
                    Ok(CommandResult {
                        exit_status: ExitStatus::Exited(n),
                        ..
                    }) => format!("{} with exit status {}.", "Failed".red(), n),
                    Ok(ref result) => format!("{} with {:?}", "Failed".red(), result.exit_status),
                    Err(ref e) => format!("{} ({:?})", "Error".red(), e),
                };
                pb.finish_with_message(&finish_msg);

                sender.send(res).unwrap();
            });
        }
        m.join().unwrap();

        results
            .iter()
            .map(|r| {
                r.recv().unwrap()
                    // TODO: Error should contain the command.
                    .map_err(|e| Error::with_chain(e, ErrorKind::FailedToRunCommand("<nyi>".to_owned())))
            })
            .collect()
    }

    pub fn output_results(
        output_type: OutputType,
        show_all: bool,
        results: &[CommandResult],
    ) -> Result<()> {
        let mut stdout = ::std::io::stdout();

        match output_type {
            OutputType::Human => {
                let output = TableOutputCommandResults { show_all };

                output
                    .output(&mut stdout, results)
                    .chain_err(|| ErrorKind::FailedToOutput)
            }
            OutputType::Json => {
                let output = JsonOutputCommandResults;

                output
                    .output(&mut stdout, results)
                    .chain_err(|| ErrorKind::FailedToOutput)
            }
            OutputType::Plain => {
                unimplemented!("'Plain' output is not supported for this module");
            }
        }
    }
}

pub mod ssh {
    use super::*;

    use std::time::Duration;
    use tempfile;

    use provider::InstanceDescriptor;
    use utils::command::Command;

    pub fn exec_ssh_to_ip_address<T: Into<IpAddr>>(
        ip: T,
        command: Option<&str>,
        ssh_opts: Option<Vec<String>>,
    ) -> Result<()> {
        use std::os::unix::process::CommandExt;

        let ip_addr: IpAddr = ip.into();

        let mut ssh_command = ::std::process::Command::new("ssh");
        let ssh_ip = ssh_command.arg(ip_addr.to_string());

        let ssh_options = if let Some(opts) = ssh_opts {
            ssh_ip.args(opts)
        } else {
            ssh_ip
        };

        let ssh_command = if let Some(c) = command {
            ssh_options.arg(c)
        } else {
            ssh_options
        };

        debug!("Exec '{:#?}'; replacing ceres now.", &ssh_command);
        let err = ssh_command.exec();

        Err(Error::with_chain(err, ErrorKind::FailedToExecuteSsh))
    }

    pub fn build_ssh_command_to_instances(
        instances: &[InstanceDescriptor],
        use_public_ip: bool,
        login_name: Option<&String>,
        ssh_opts: &[&str],
        remote_commands_args: &[&str],
        timeout: Duration,
    ) -> Result<Vec<Command>> {
        let commands: Result<Vec<_>> = instances
            .iter()
            .map(|i| {
                let ip_addr: IpAddr = if use_public_ip {
                    i.public_ip_address.as_ref()
                } else {
                    i.private_ip_address.as_ref()
                }
                    .map(|ip| ip.parse())
                    // TODO Fix me!
                    .chain_err(|| ErrorKind::FailedToBuildSshCommand)?
                    .chain_err(|| ErrorKind::FailedToBuildSshCommand)?;
                let instance_id = i
                    .instance_id
                    .as_ref()
                    .chain_err(|| ErrorKind::FailedToBuildSshCommand)?;
                let command = build_ssh_command_to_instance(
                    &instance_id,
                    &ip_addr,
                    login_name,
                    &ssh_opts,
                    &remote_commands_args,
                    timeout,
                )?;
                trace!("ssh_args for instance {}: {:#?}", instance_id, command);
                Ok(command)
            })
            .collect();

        commands
    }

    pub fn build_ssh_command_to_instance(
        instance_id: &str,
        ip_addr: &IpAddr,
        login_name: Option<&String>,
        ssh_opts: &[&str],
        remote_command_args: &[&str],
        timeout: Duration,
    ) -> Result<Command> {
        let mut ssh_opts: Vec<String> = ssh_opts.iter().map(|s| s.to_string()).collect();
        if let Some(login_name) = login_name {
            ssh_opts.insert(0, "-l".to_owned());
            ssh_opts.insert(1, login_name.to_owned());
        };

        let mut remote_command_args: Vec<String> =
            remote_command_args.iter().map(|s| s.to_string()).collect();

        let ssh_args = build_ssh_arguments(&ip_addr, &mut ssh_opts, &mut remote_command_args);

        let log_path = tempfile::NamedTempFile::new()
            .chain_err(|| ErrorKind::FailedToBuildSshCommand)?
            .path()
            .to_path_buf();
        let c = Command {
            id: instance_id.to_owned(),
            cmd: "ssh".to_owned(),
            args: Some(ssh_args),
            cwd: None,
            log: log_path,
            timeout: Some(timeout),
        };
        Ok(c)
    }

    fn build_ssh_arguments(
        ip_addr: &IpAddr,
        ssh_opts: &mut Vec<String>,
        remote_command_args: &mut Vec<String>,
    ) -> Vec<String> {
        let mut ssh_args = Vec::new();

        ssh_args.append(ssh_opts);
        ssh_args.push(ip_addr.to_string());
        ssh_args.append(remote_command_args);

        ssh_args
    }
}

error_chain! {
    errors {
        FailedToBuildSshCommand {
            description("Failed to build ssh command")
        }
        FailedToExecuteSsh {
            description("Failed to execute ssh")
        }
        FailedToInitLogging {
            description("Failed to init logging framework")
        }
        FailedToRunCommand(cmd: String) {
            description("Failed to run command")
            display("Failed to run command '{}'", cmd)
        }
        FailedToRunCommands {
            description("Failed to run commands")
        }
        FailedToOutput{
            description("Failed to output")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };
    use tempfile::NamedTempFile;

    #[test]
    fn run_non_existing_command() {
        let tmpfile = NamedTempFile::new().unwrap().path().to_path_buf();

        let cmd = command::Command {
            id: "a command".to_owned(),
            cmd: "this_command_does_not_exists".to_owned(),
            args: None,
            cwd: None,
            log: tmpfile,
            timeout: None,
        };
        let res = cmd.run(None::<fn()>);

        assert_that(&res).is_err();
    }

    #[test]
    fn run_command_successfully() {
        let tmpfile = NamedTempFile::new().unwrap().path().to_path_buf();

        let cmd = command::Command {
            id: "ls".to_owned(),
            cmd: "/bin/ls".to_owned(),
            args: None,
            cwd: None,
            log: tmpfile,
            timeout: None,
        };
        let res = cmd.run(None::<fn()>);

        assert_that(&res).is_ok();
    }

    #[test]
    fn run_command_successfully_and_check_log_file() {
        let tmpfile = NamedTempFile::new().unwrap().path().to_path_buf();

        let cmd = command::Command {
            id: "ls".to_owned(),
            cmd: "/bin/ls".to_owned(),
            args: Some(vec![
                "-l".to_owned(),
                "LICENSE".to_owned(),
                "Makefile".to_owned(),
            ]),
            cwd: None,
            log: tmpfile.clone(),
            timeout: None,
        };
        let res = cmd.run(None::<fn()>);

        assert_that(&res).is_ok();

        let output = BufReader::new(File::open(tmpfile).unwrap());
        assert_that(&output.lines().count()).is_equal_to(2);
    }
}
