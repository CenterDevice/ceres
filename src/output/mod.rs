use std::io::Write;

use provider::InstanceDescriptor;

pub mod json_output;
pub mod table_output;

pub trait OutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()>;
}

error_chain! {
    errors {
        OutputFailed {
            description("Failed to output.")
        }
    }
}