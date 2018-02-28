use prettytable::Table;
use prettytable::cell::Cell;
use prettytable::format;
use prettytable::row::Row;
use std::collections::HashMap;
use std::io::Write;

use provider::{InstanceDescriptor, InstanceDescriptorFields};
use output::*;

pub struct TableOutputInstances {
    pub fields: Vec<InstanceDescriptorFields>,
}

impl Default for TableOutputInstances {
    fn default() -> Self {
        TableOutputInstances {
            fields: vec![
                InstanceDescriptorFields::InstanceId,
                InstanceDescriptorFields::InstanceType,
                InstanceDescriptorFields::LaunchTime,
                InstanceDescriptorFields::PrivateIpAddress,
                InstanceDescriptorFields::PublicIpAddress,
            ]
        }
    }
}

impl OutputInstances for TableOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(
            Row::new(
                self.fields.iter().map(|f| Cell::new(header_for_field(f))).collect::<Vec<_>>()
            ));

        // We have to create / allocate the Strings first since `Table` only accepts `&str` and some
        // `InstanceDescriptorFields` need to allocate representations first, e.g., `InstanceDescriptorFields::Tags`
        let mut rows = Vec::new();
        for instance in instances {
            let row = self.fields.iter().map(|f| value_for_field(f, instance)).collect::<Vec<_>>();
            rows.push(row);
        }
        for r in rows {
            table.add_row(
                Row::new(
                    r.iter().map(|cell| Cell::new(cell)).collect::<Vec<_>>()
                ));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn header_for_field(field: &InstanceDescriptorFields) -> &str {
    match *field {
        InstanceDescriptorFields::Hypervisor => "Hypervisor",
        InstanceDescriptorFields::InstanceId => "Instance Id",
        InstanceDescriptorFields::InstanceType => "Instance Type",
        InstanceDescriptorFields::LaunchTime => "Launch Time",
        InstanceDescriptorFields::PrivateDnsName => "Private DNS Name",
        InstanceDescriptorFields::PrivateIpAddress => "Private IP Address",
        InstanceDescriptorFields::PublicDnsName => "Public DNS Name",
        InstanceDescriptorFields::PublicIpAddress => "Public IP Address",
        InstanceDescriptorFields::RootDeviceName => "Root Device Name",
        InstanceDescriptorFields::RootDeviceType => "Root Device Type",
        InstanceDescriptorFields::Tags => "Tags",
    }
}

fn value_for_field(field: &InstanceDescriptorFields, instance: &InstanceDescriptor) -> String {
    match *field {
        InstanceDescriptorFields::Hypervisor => instance.hypervisor.clone(),
        InstanceDescriptorFields::InstanceId => instance.instance_id.clone(),
        InstanceDescriptorFields::InstanceType => instance.instance_type.clone(),
        InstanceDescriptorFields::LaunchTime => instance.launch_time.clone(),
        InstanceDescriptorFields::PrivateDnsName => instance.private_dns_name.clone(),
        InstanceDescriptorFields::PrivateIpAddress => instance.private_ip_address.clone(),
        InstanceDescriptorFields::PublicDnsName => instance.public_dns_name.clone(),
        InstanceDescriptorFields::PublicIpAddress => instance.public_ip_address.clone(),
        InstanceDescriptorFields::RootDeviceName => instance.root_device_name.clone(),
        InstanceDescriptorFields::RootDeviceType => instance.root_device_type.clone(),
        InstanceDescriptorFields::Tags =>
            Some(format_tags(instance.tags.as_ref().unwrap())),
    }.unwrap_or_else(|| String::from("-"))
}

/// Format a `HashMap` of `String` -> `Option<String>` into a single line, pretty string.
fn format_tags(tags: &HashMap<String, Option<String>>) -> String {
    let empty = String::from("");
    let mut concat = String::new();
    let mut iter = tags.iter();
    if let Some((k, v)) = iter.next() {
        concat.push_str(k);
        concat.push_str(":");
        concat.push_str(v.as_ref().unwrap_or(&empty));
    };
    for (k, v) in iter {
        concat.push_str(", ");
        concat.push_str(k);
        concat.push_str(":");
        concat.push_str(v.as_ref().unwrap_or(&empty));
    }
    concat
}

#[cfg(test)]
mod test {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn format_tags_empty() {
        let tags = HashMap::new();

        let res = format_tags(&tags);

        let expected = String::from("");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_one_kv() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));

        let res = format_tags(&tags);

        let expected = String::from("key1:value1");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_multiple_kv() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value2".to_owned()));
        tags.insert("key2".to_owned(), None);
        tags.insert("key3".to_owned(), Some("value2".to_owned()));

        let res = format_tags(&tags);

        let expected = String::from("key2:, key3:value2, key1:value2");
        assert_that(&res).is_equal_to(&expected);
    }
}
