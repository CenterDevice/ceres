use std::io::{self, BufRead, BufReader, Write};
use std::net::IpAddr;
use std::process::Command;
use std::os::unix::process::CommandExt;

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
        Err(e) => Err(Error::with_chain(e, ErrorKind::FailedToReadFromStdin))
    }
}

pub fn ssh_to_ip_address<T: Into<IpAddr>>(ip: T, command: Option<&str>, ssh_opts: Option<&str>) -> Result<()> {
    let ip_addr: IpAddr = ip.into();

    let mut ssh_command = Command::new("ssh");
    let ssh_ip = ssh_command.arg(ip_addr.to_string());

    let ssh_options = if let Some(opts) = ssh_opts {
        ssh_ip.args(opts.split(" "))
    } else {
        ssh_ip
    };

    let ssh_command = if let Some(c) = command {
        ssh_options.arg(c)
    } else {
        ssh_options
    };
    debug!("Executing '{:#?}'", &ssh_command);

    let err = ssh_command.exec();
    Err(Error::with_chain(err, ErrorKind::FailedToExecuteSsh))
}

error_chain! {
    errors {
        FailedToReadFromStdin {
            description("Failed to read from stdin")
        }
        FailedToExecuteSsh {
            description("Failed to execute ssh")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::{TestResult, quickcheck};
    use spectral::prelude::*;

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
                return TestResult::discard()
            }

            let mut buf = BufReader::new(x.as_bytes());
            let res = ask_for_yes_from_reader(&mut buf, "This is just a test prompt: ").unwrap();
            TestResult::from_bool(res == false)
        }

        quickcheck(prop as fn(String) -> TestResult);
    }
}
