use serde_json;

use output::stories::*;

pub struct JsonOutputStory;

impl OutputStory for JsonOutputStory {
    fn output<T: Write>(&self, writer: &mut T, story: &Story, _: &[ProjectMember]) -> Result<()> {
        serde_json::to_writer_pretty(writer, story).chain_err(|| ErrorKind::OutputFailed)
    }
}

