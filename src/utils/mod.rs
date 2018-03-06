use std::io::{self, BufReader, Read};

pub fn read_for_yes_from_stdin(prompt: &str) -> Result<bool> {
    let reader = io::stdin();
    read_for_yes(&reader, prompt)
}

fn read_for_yes<T: Read>(reader: &T,prompt: &str) -> Result<bool> {
    let mut reader = BufReader::new(reader);
    print!("{}", prompt);

    let mut input = String::new();
    match reader.read_line(&mut input) {
        Ok(_) => {
            if input.as_slice().lowercase() == "yes" {
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

