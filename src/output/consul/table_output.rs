use prettytable::{cell::Cell, format, row::Row, Table};
use service_world::consul::Catalog;
use std::io::Write;

use modules::consul::NodeField;
use output::consul::*;

pub struct TableOutputCatalogResult {
    pub fields: Vec<NodeField>,
}

impl Default for TableOutputCatalogResult {
    fn default() -> Self {
        TableOutputCatalogResult {
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
            ],
        }
    }
}

impl OutputCatalogResult for TableOutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, catalog: &Catalog) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        // TODO: Add health indication via catalog.is_node_healthy_for_service()
        table.set_titles(Row::new(
            self.fields
                .iter()
                .map(|f| Cell::new(header_for_field(f)))
                .collect::<Vec<_>>(),
        ));

        // We have to create / allocate the Strings first since `Table` only accepts `&str` and some
        // `InstanceDescriptorFields` need to allocate representations first, e.g.,
        // `InstanceDescriptorFields::Tags`
        let mut rows = Vec::new();
        for service in catalog.services() {
            if let Some(nodes) = catalog.nodes_by_service(service) {
                for node in nodes {
                    let row = self
                        .fields
                        .iter()
                        .map(|f| value_for_field(f, catalog, node))
                        .collect::<Vec<_>>();
                    rows.push(row);
                }
            }
        }
        for r in rows {
            table.add_row(Row::new(
                r.iter().map(|cell| Cell::new(cell)).collect::<Vec<_>>(),
            ));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn header_for_field(field: &NodeField) -> &str {
    match *field {
        NodeField::Id => "Node Id",
        NodeField::Name => "Node Name",
        NodeField::MetaData(_) => "Meta Data",
        NodeField::Address => "Node Address",
        NodeField::ServicePort => "Service Port",
        NodeField::ServiceTags => "Service Tags",
        NodeField::ServiceId => "Service Id",
        NodeField::ServiceName => "Service Name",
        NodeField::Healthy => "Healthy",
    }
}
