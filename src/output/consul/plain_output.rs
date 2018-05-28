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
                NodeField::Healthy,
            ]
        }
    }
}

impl OutputCatalogResult for PlainOutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, catalog: &Catalog) -> Result<()> {
        let mut rows = Vec::new();
        for service in catalog.services() {
            if let Some(nodes) = catalog.nodes_by_service(service) {
                for node in nodes {
                    match self.fields.as_slice() {
                        // cf. https://github.com/rust-lang/rust/issues/23121
                        /*
                        [NodeField::MetaData(Some(ref mdf))] if mdf.len() == 1 && mdf.first().unwrap() == "ec2_instance_id" => {
                            rows.push( node.metadata.get("ec2_instance_id").unwrap_or("n/a"));
                        },
                        */
                        _ => {
                            let row = self.fields
                                .iter()
                                .map(|f| value_for_field(f, catalog, node))
                                .collect::<Vec<_>>();
                            rows.push(row);
                        }
                    }
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

