use pivotal_api::{ProjectMember, Story};

use std::{io::Write, str::FromStr};

pub enum OutputType {
    Json,
    MarkDown,
}

impl FromStr for OutputType {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s.to_owned().to_uppercase().as_ref() {
            "JSON" => Ok(OutputType::Json),
            "MARKDOWN" => Ok(OutputType::MarkDown),
            _ => Err(Error::from_kind(ErrorKind::OutputParsingFailed(s.to_owned()))),
        }
    }
}

pub mod json_output;
pub mod markdown_output;

pub use self::{json_output::JsonOutputStory, markdown_output::MarkDownOutputStory};

pub trait OutputStory {
    fn output<T: Write>(&self, writer: &mut T, story: &Story, members: &[ProjectMember]) -> Result<()>;
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
