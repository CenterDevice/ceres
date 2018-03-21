use log;
use fern;
use fern::colors::{Color, ColoredLevelConfig};
use std::time::Duration;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net::IpAddr;
use std::process::Command;
use std::os::unix::process::CommandExt;
use subprocess::{Exec, ExitStatus, Redirection};

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

pub fn run_commmand<T: AsRef<OsStr>>(
    cmd: &str,
    args: Option<&[T]>,
    log: File,
    timeout: Option<u64>)
-> Result<ExitStatus> {
    let mut p = if let Some(args) = args {
        Exec::cmd(cmd)
        .args(args)
    } else {
        Exec::cmd(cmd)
    }
        .stdout(log)
        .stderr(Redirection::Merge)
        .popen()
        .chain_err(|| ErrorKind::FailedToRunCommand(cmd.to_owned()))?;

    let res = if let Some(timeout) = timeout {
        if let Some(status) = p.wait_timeout(Duration::new(timeout, 0))
                .chain_err(|| ErrorKind::FailedToRunCommand(cmd.to_owned()))? {
            println!("process finished as {:?}", status);
            status
        } else {
            p.kill().chain_err(|| ErrorKind::FailedToRunCommand(cmd.to_owned()))?;
            let res = p.wait().chain_err(|| ErrorKind::FailedToRunCommand(cmd.to_owned()))?;
            println!("process killed");
            res
        }
    } else {
        p.wait().chain_err(|| ErrorKind::FailedToRunCommand(cmd.to_owned()))?
    };

    Ok(res)
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
    use std::io::BufReader;
    use spectral::prelude::*;
    use tempfile::{self, NamedTempFile};

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
        let tmpfile: File = tempfile::tempfile().unwrap();

        let res = run_commmand("this_command_does_not_exists", None::<&[&str]>, tmpfile, None);

        assert_that(&res).is_err();
    }

    #[test]
    fn run_command_successfully() {
        let tmpfile: File = tempfile::tempfile().unwrap();

        let res = run_commmand("/bin/ls", None::<&[&str]>, tmpfile, None);

        assert_that(&res).is_ok();
    }

    #[test]
    fn run_command_successfully_and_check_log_file() {
        let tmpfile = NamedTempFile::new().unwrap();

        let res = run_commmand("/bin/ls", Some(&["-l", "LICENSE", "Makefile"]), tmpfile.reopen().unwrap(), None);

        assert_that(&res).is_ok();

        let output = BufReader::new(&tmpfile);
        assert_that(&output.lines().count()).is_equal_to(2);
    }
}
