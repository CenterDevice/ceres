use std::io::Write;

use provider::{InstanceDescriptor, StateChange};

pub mod instances;

pub trait OutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()>;
}

pub trait OutputStateChanges {
    fn output<T: Write>(&self, writer: &mut T, state_changes: &[StateChange]) -> Result<()>;
}

error_chain! {
    errors {
        OutputFailed {
            description("Failed to output.")
        }
    }
}
