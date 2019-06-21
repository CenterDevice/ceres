use std::io::Write;

use output::*;
use provider::{InstanceDescriptor, StateChange};
use utils::command::CommandResult;

pub mod json_output;
pub mod plain_output;
pub mod table_output;

pub use self::{
    json_output::{JsonOutputCommandResults, JsonOutputInstances, JsonOutputStateChanges},
    plain_output::PlainOutputInstances,
    table_output::{TableOutputCommandResults, TableOutputInstances, TableOutputStatusChanges},
};

pub trait OutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()>;
}

pub trait OutputStateChanges {
    fn output<T: Write>(&self, writer: &mut T, state_changes: &[StateChange]) -> Result<()>;
}

pub trait OutputCommandResults {
    fn output<T: Write>(&self, writer: &mut T, results: &[CommandResult]) -> Result<()>;
}
