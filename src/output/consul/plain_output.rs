use service_world::consul::Catalog;
use std::io::Write;

use output::consul::*;

pub struct PlainOutputCatalogResult {
    pub fields: Vec<NodeField>,
}

impl Default for PlainOutputCatalogResult {
    fn default() -> Self {
        PlainOutputCatalogResult {
            fields: vec![
                NodeField::Id,
                NodeField::Name,
                NodeField::MetaData(None),
                NodeField::Address,
                NodeField::ServicePort,
                NodeField::ServiceTags,
                NodeField::ServiceId,
                NodeField::ServiceName,
            ]
        }
    }
}

impl OutputCatalogResult for PlainOutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, catalog: &Catalog) -> Result<()> {
        // We have to create / allocate the Strings first since `Table` only accepts `&str` and some
        // `InstanceDescriptorFields` need to allocate representations first, e.g., `InstanceDescriptorFields::Tags`
        let mut rows = Vec::new();
        for service in catalog.services() {
            if let Some(nodes) = catalog.nodes_by_service(service) {
                for node in nodes {
                    let row = self.fields
                        .iter()
                        .map(|f| value_for_field(f, node))
                        .collect::<Vec<_>>();
                    rows.push(row);
                }
            }
        }
        for r in rows {
            let line = format!("{}\n", r.as_slice().join(";"));
            let _ = writer.write(line.as_bytes());
        }

        Ok(())
    }
}

