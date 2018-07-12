use service_world::consul::{Catalog, Node};
use std::collections::HashMap;
use std::io::Write;

use modules::consul::NodeField;
use output::*;

pub mod json_output;
pub mod plain_output;
pub mod table_output;

pub use self::json_output::JsonOutputCatalogResult;
pub use self::plain_output::PlainOutputCatalogResult;
pub use self::table_output::TableOutputCatalogResult;

pub trait OutputCatalogResult {
    fn output<T: Write>(&self, writer: &mut T, results: &Catalog) -> Result<()>;
}

fn value_for_field(field: &NodeField, catalog: &Catalog, node: &Node) -> String {
    match *field {
        NodeField::Id => node.id.clone(),
        NodeField::Name => node.name.clone(),
        NodeField::MetaData(ref filter) => {
            format_meta_data(&node.meta_data, filter.as_ref().map(|x| x.as_slice()))
        }
        NodeField::Address => node.address.clone(),
        NodeField::ServicePort => node.service_port.to_string(),
        NodeField::ServiceTags => node.service_tags.as_slice().join(","),
        NodeField::ServiceId => node.service_id.clone(),
        NodeField::ServiceName => node.service_name.clone(),
        NodeField::Healthy => is_healthy(catalog, node),
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

fn is_healthy(catalog: &Catalog, node: &Node) -> String {
    catalog
        .is_node_healthy_for_service(node, &node.service_name)
        .to_string()
}
