use std::str::FromStr;

pub mod centerdevice;
pub mod consul;
pub mod health;
pub mod infrastructure;
pub mod instances;
pub mod statuspages;

pub enum OutputType {
    Human,
    Json,
    Plain,
}

impl FromStr for OutputType {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s.to_owned().to_uppercase().as_ref() {
            "HUMAN" => Ok(OutputType::Human),
            "JSON" => Ok(OutputType::Json),
            "PLAIN" => Ok(OutputType::Plain),
            _ => Err(Error::from_kind(ErrorKind::OutputParsingFailed(
                s.to_owned(),
            ))),
        }
    }
}

error_chain! {
    errors {
        OutputParsingFailed(s: String) {
            description("Failed to parse Output from String.")
            display("Failed to parse Output from String '{}'.", s)
        }
        OutputFailed {
            description("Failed to output.")
        }
    }
}
