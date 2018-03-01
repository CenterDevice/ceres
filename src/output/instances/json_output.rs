use serde_json;

use output::*;
use provider::InstanceDescriptor;

pub struct JsonOutputInstances;

impl OutputInstances for JsonOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        serde_json::to_writer_pretty(writer, instances)
            .chain_err(|| ErrorKind::OutputFailed)
    }
}
