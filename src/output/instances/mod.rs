use std::io::Write;

use provider::{InstanceDescriptor, StateChange};
use output::*;
use utils::command::CommandResult;

pub mod json_output;
pub mod plain_output;
pub mod table_output;

pub use self::json_output::{JsonOutputCommandResults, JsonOutputInstances, JsonOutputStateChanges};
pub use self::plain_output::{PlainOutputInstances};
pub use self::table_output::{TableOutputCommandResults, TableOutputInstances, TableOutputStatusChanges};

pub trait OutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()>;
}

pub trait OutputStateChanges {
    fn output<T: Write>(&self, writer: &mut T, state_changes: &[StateChange]) -> Result<()>;
}

pub trait OutputCommandResults {
    fn output<T: Write>(&self, writer: &mut T, results: &[CommandResult]) -> Result<()>;
}

