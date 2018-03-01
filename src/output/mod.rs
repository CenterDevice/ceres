use std::io::Write;

use provider::InstanceDescriptor;

pub mod instances;

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