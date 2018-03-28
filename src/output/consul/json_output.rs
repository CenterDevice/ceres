use serde_json;

use output::consul::*;
use service_world::consul::Catalog;

pub struct JsonOutputCatalogResult;

impl OutputCatalogResult for JsonOutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, result: &Catalog) -> Result<()> {
        serde_json::to_writer_pretty(writer, result).chain_err(|| ErrorKind::OutputFailed)
    }
}
