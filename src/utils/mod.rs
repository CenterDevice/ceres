use log;
use fern;
use fern::colors::{Color, ColoredLevelConfig};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::Command;
use std::os::unix::process::CommandExt;
use tail;

pub fn ask_for_yes_from_stdin(prompt: &str) -> Result<bool> {
    let mut reader = BufReader::new(io::stdin());
    ask_for_yes_from_reader(&mut reader, prompt)
}

fn ask_for_yes_from_reader<R: BufRead>(reader: &mut R, prompt: &str) -> Result<bool> {
    print!("{}", prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();
    match reader.read_line(&mut input) {
        Ok(_) => {
            if input.trim().to_lowercase() == "yes" {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => Err(Error::with_chain(e, ErrorKind::FailedToReadFromStdin)),
    }
}

pub fn init_logging(ceres: log::LevelFilter, default: log::LevelFilter) -> Result<()> {
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::Blue);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let level = format!("{}", record.level());
            out.finish(format_args!(
                "{}{:padding$}{}: {}",
                colors.color(record.level()),
                " ",
                record.target(),
                message,
                padding = 6 - level.len(),
            ))
        })
        .chain(
            fern::Dispatch::new()
                .level(default)
                .level_for("ceres", ceres)
                .chain(io::stderr()),
        )
        .apply()
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToInitLogging))?;

    Ok(())
}

pub fn int_to_log_level(n: u64) -> log::LevelFilter {
    match n {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}

pub fn ssh_to_ip_address<T: Into<IpAddr>>(
    ip: T,
    command: Option<&str>,
    ssh_opts: Option<&str>,
) -> Result<()> {
    let ip_addr: IpAddr = ip.into();

    let mut ssh_command = Command::new("ssh");
    let ssh_ip = ssh_command.arg(ip_addr.to_string());

    let ssh_options = if let Some(opts) = ssh_opts {
        ssh_ip.args(opts.split(' '))
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

pub mod command {
    use super::*;

    use std::time::Duration;
    use std::fs::File;
    use subprocess::{Exec, ExitStatus as SubprocessExitStatus, Redirection};

    #[derive(Debug)]
    pub struct Command {
        pub id: String,
        pub cmd: String,
        pub args: Option<Vec<String>>,
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
            let mut p = if let Some(ref args) = self.args {
                Exec::cmd(&cmd)
                .args(args)
            } else {
                Exec::cmd(&cmd)
            }
                .stdout(File::create(self.log.clone()).unwrap())
                .stderr(Redirection::Merge)
                .popen()
                .chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;

            let mut timeout = self.timeout;
            let resolution = Duration::from_millis(100);
            loop {
                let status = p.wait_timeout(resolution)
                        .chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;

                if let Some(count_down) = timeout {
                    let count_down = count_down - resolution;
                    if count_down <= Duration::from_secs(0) {
                        p.kill().chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;
                        let exit_status = p.wait().chain_err(|| ErrorKind::FailedToRunCommand(cmd.clone()))?;
                        return Ok( CommandResult{ id: self.id, log: self.log, exit_status: exit_status.into() } )
                    }
                    timeout = Some(count_down);
                }

                if let Some(ref progress) = progress {
                    progress();
                }
                match status {
                    Some(exit_status) => return Ok( CommandResult{ id: self.id, log: self.log, exit_status: exit_status.into() } ),
                    None => {},
                }
            }
        }
    }
}

pub trait FileExt {
    fn read_last_line(self) -> ::std::io::Result<String>;
}

impl FileExt for File {
    fn read_last_line(self) -> ::std::io::Result<String> {
        let mut fd = BufReader::new(self);
        let mut reader = tail::BackwardsReader::new(10, &mut fd);
        let mut buffer = String::new();
        {
            let mut writer = BufWriter::new(
                unsafe {
                    buffer.as_mut_vec()
                }
            );
            reader.read_all(&mut writer);
        }
        let line = buffer.lines().last().map(|s| s.to_owned()).unwrap_or_else(|| String::new());
        Ok(line)
    }
}

error_chain! {
    errors {
        FailedToReadFromStdin {
            description("Failed to read from stdin")
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::{quickcheck, TestResult};
    use std::fs::File;
    use std::io::BufReader;
    use spectral::prelude::*;
    use tempfile::NamedTempFile;

    #[test]
    fn ask_for_yes_from_reader_okay_lowercase() {
        let answer = "yes".to_owned();
        let mut buf = BufReader::new(answer.as_bytes());
        let res = ask_for_yes_from_reader(&mut buf, "This is just a test prompt: ");

        assert_that(&res).is_ok().is_true();
    }

    #[test]
    fn ask_for_yes_from_reader_okay_uppercase() {
        let answer = "YES".to_owned();
        let mut buf = BufReader::new(answer.as_bytes());
        let res = ask_for_yes_from_reader(&mut buf, "This is just a test prompt: ");

        assert_that(&res).is_ok().is_true();
    }

    #[test]
    fn ask_for_yes_from_reader_okay_mixedcase() {
        let answer = "YeS".to_owned();
        let mut buf = BufReader::new(answer.as_bytes());
        let res = ask_for_yes_from_reader(&mut buf, "This is just a test prompt: ");

        assert_that(&res).is_ok().is_true();
    }

    #[test]
    fn ask_for_yes_from_reader_quick() {
        fn prop(x: String) -> TestResult {
            if x.len() > 3 || x.to_lowercase() == "yes" {
                return TestResult::discard();
            }

            let mut buf = BufReader::new(x.as_bytes());
            let res = ask_for_yes_from_reader(&mut buf, "This is just a test prompt: ").unwrap();
            TestResult::from_bool(res == false)
        }

        quickcheck(prop as fn(String) -> TestResult);
    }

    #[test]
    fn run_non_existing_command() {
        let tmpfile = NamedTempFile::new().unwrap().path().to_path_buf();

        let cmd = command::Command {
            id: "a command".to_owned(),
            cmd: "this_command_does_not_exists".to_owned(),
            args: None,
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
            args: Some(vec!["-l".to_owned(), "LICENSE".to_owned(), "Makefile".to_owned()]),
            log: tmpfile.clone(),
            timeout: None,
        };
        let res = cmd.run(None::<fn()>);

        assert_that(&res).is_ok();

        let output = BufReader::new(File::open(tmpfile).unwrap());
        assert_that(&output.lines().count()).is_equal_to(2);
    }
}
