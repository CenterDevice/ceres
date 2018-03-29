use prettytable::Table;
use prettytable::cell::Cell;
use prettytable::format;
use prettytable::row::Row;
use service_world::consul::{Catalog, Node};
use std::collections::HashMap;
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
            ]
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
            table.add_row(Row::new(
                r.iter().map(|cell| Cell::new(cell)).collect::<Vec<_>>(),
            ));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn header_for_field(field: &NodeField) -> &str {
    match *field {
        NodeField::Id => "Id",
        NodeField::Name => "Name",
        NodeField::MetaData(_) => "Meta Data",
        NodeField::Address => "Address",
        NodeField::ServicePort => "Service Port",
        NodeField::ServiceTags => "Service Tags",
        NodeField::ServiceId => "Service Id",
        NodeField::ServiceName => "Service Name",
    }
}

fn value_for_field(field: &NodeField, node: &Node) -> String {
    match *field {
        NodeField::Id => node.id.clone(),
        NodeField::Name => node.name.clone(),
        NodeField::MetaData(ref filter) => format_meta_data(
            &node.meta_data,
            filter.as_ref().map(|x| x.as_slice())
        ),
        NodeField::Address => node.address.clone(),
        NodeField::ServicePort => node.service_port.to_string(),
        NodeField::ServiceTags =>  node.service_tags.as_slice().join(","),
        NodeField::ServiceId => node.service_id.clone(),
        NodeField::ServiceName => node.service_name.clone(),
    }
}

/// Format a `HashMap` of `String` -> `String` into a single line, pretty string.
fn format_meta_data(tags: &HashMap<String, String>, filter: Option<&[String]>) -> String {
    let mut concat = String::new();

    let mut keys: Vec<_> = if let Some(filter) = filter {
        tags.keys().filter(|&k| filter.contains(k)).collect()
    } else {
        tags.keys().collect()
    };
    keys.sort();
    let mut iter = keys.into_iter();

    if let Some(k) = iter.next() {
        concat.push_str(k);
        concat.push_str("=");
        concat.push_str(tags.get(k).unwrap().as_ref());
    };
    for k in iter {
        concat.push_str(", ");
        concat.push_str(k);
        concat.push_str("=");
        concat.push_str(tags.get(k).unwrap().as_ref());
    }
    concat
}
