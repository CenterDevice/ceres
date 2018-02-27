use std::collections::HashMap;
use std::io::Write;

use provider::{InstanceDescriptor, InstanceDescriptorFields};
use output::*;

pub struct TableOutputInstances {
    fields: Vec<InstanceDescriptorFields>,
}

impl Default for TableOutputInstances {
    fn default() -> Self {
        TableOutputInstances {
            fields: vec![
                InstanceDescriptorFields::InstanceId,
                InstanceDescriptorFields::InstanceType,
                InstanceDescriptorFields::PrivateDnsName,
                InstanceDescriptorFields::Tags,
            ]
        }
    }
}

impl OutputInstances for TableOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        for instance in instances {
            for field in &self.fields {
                let field_str = match *field {
                    InstanceDescriptorFields::Hypervisor => format!("Hypervisor: {}", instance.hypervisor.as_ref().unwrap()),
                    InstanceDescriptorFields::InstanceId => format!("Instance Id: {}", instance.instance_id.as_ref().unwrap()),
                    InstanceDescriptorFields::InstanceType => format!("Instance Type: {}", instance.instance_type.as_ref().unwrap()),
                    InstanceDescriptorFields::LaunchTime => format!("Launch Time: {}", instance.launch_time.as_ref().unwrap()),
                    InstanceDescriptorFields::PrivateDnsName => format!("Private DNS Name: {}", instance.private_dns_name.as_ref().unwrap()),
                    InstanceDescriptorFields::PrivateIpAddress => format!("Private IP Address: {}", instance.private_ip_address.as_ref().unwrap()),
                    InstanceDescriptorFields::PublicDnsName => format!("Public DNS Name: {}", instance.public_dns_name.as_ref().unwrap()),
                    InstanceDescriptorFields::PublicIpAddress => format!("Public IP Address: {}", instance.public_dns_name.as_ref().unwrap()),
                    InstanceDescriptorFields::RootDeviceName => format!("Root Device Name: {}", instance.root_device_name.as_ref().unwrap()),
                    InstanceDescriptorFields::RootDeviceType => format!("Root Device Type: {}", instance.root_device_type.as_ref().unwrap()),
                    InstanceDescriptorFields::Tags => format!("Tags: {:?}", format_tags(instance.tags.as_ref().unwrap())),
                };
                write!(writer, "{} ", field_str).chain_err(|| ErrorKind::OutputFailed)?;
            }
            writeln!(writer, "").chain_err(|| ErrorKind::OutputFailed)?;
        }
        Ok(())
    }
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
