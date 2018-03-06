use std::io::{self, BufRead, BufReader, Write};

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

error_chain! {
    errors {
        FailedToReadFromStdin {
            description("Failed to read from stdin")
        }
    }
}

