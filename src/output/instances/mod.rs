use std::str::FromStr;

pub mod json_output;
pub mod table_output;

pub use self::json_output::JsonOutputInstances;
pub use self::table_output::TableOutputInstances;

pub enum OutputType {
    Json,
    Human,
}

impl FromStr for OutputType {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s.to_owned().to_uppercase().as_ref() {
            "JSON" => Ok(OutputType::Json),
            "HUMAN" => Ok(OutputType::Human),
            _ => Err(Error::from_kind(ErrorKind::OutputParsingFailed(s.to_owned()))),
        }
    }
}

error_chain! {
    errors {
        OutputParsingFailed(s: String) {
            description("Failed to parse Output from String.")
            display("Failed to parse Output from String '{}'.", s)
        }
    }
}
