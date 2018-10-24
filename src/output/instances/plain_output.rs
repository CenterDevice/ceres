use std::collections::HashMap;
use std::io::Write;

use output::instances::*;
use provider::{InstanceDescriptor, InstanceDescriptorFields};

pub struct PlainOutputInstances {
    pub fields: Vec<InstanceDescriptorFields>,
}

impl Default for PlainOutputInstances {
    fn default() -> Self {
        PlainOutputInstances {
            fields: vec![
                InstanceDescriptorFields::InstanceId,
                InstanceDescriptorFields::InstanceType,
                InstanceDescriptorFields::State,
                InstanceDescriptorFields::PrivateIpAddress,
                InstanceDescriptorFields::PublicIpAddress,
                InstanceDescriptorFields::LaunchTime,
            ],
        }
    }
}

impl OutputInstances for PlainOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        let mut rows = Vec::new();
        for instance in instances {
            match self.fields.as_slice() {
                _ => {
                    let row = self
                        .fields
                        .iter()
                        .map(|f| value_for_field(f, instance))
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

fn value_for_field(field: &InstanceDescriptorFields, instance: &InstanceDescriptor) -> String {
    match *field {
        InstanceDescriptorFields::BlockDeviceMappings => instance
            .block_device_mappings
            .as_ref()
            .map(|bdms| bdms.join("\n")),
        InstanceDescriptorFields::Hypervisor => instance.hypervisor.clone(),
        InstanceDescriptorFields::IamInstanceProfile => instance.iam_instance_profile.clone(),
        InstanceDescriptorFields::ImageId => instance.image_id.clone(),
        InstanceDescriptorFields::InstanceId => instance.instance_id.clone(),
        InstanceDescriptorFields::InstanceType => instance.instance_type.clone(),
        InstanceDescriptorFields::LaunchTime => instance.launch_time.clone(),
        InstanceDescriptorFields::Monitoring => instance.monitoring.clone(),
        InstanceDescriptorFields::Placement => instance.placement.clone(),
        InstanceDescriptorFields::PrivateDnsName => instance.private_dns_name.clone(),
        InstanceDescriptorFields::PrivateIpAddress => instance.private_ip_address.clone(),
        InstanceDescriptorFields::PublicDnsName => instance.public_dns_name.clone(),
        InstanceDescriptorFields::PublicIpAddress => instance.public_ip_address.clone(),
        InstanceDescriptorFields::RootDeviceName => instance.root_device_name.clone(),
        InstanceDescriptorFields::RootDeviceType => instance.root_device_type.clone(),
        InstanceDescriptorFields::SecurityGroups => {
            instance.security_groups.as_ref().map(|sgs| sgs.join("\n"))
        }
        InstanceDescriptorFields::State => instance.state.clone(),
        InstanceDescriptorFields::StateReason => instance.state_reason.clone(),
        InstanceDescriptorFields::Tags(ref tags_filter) => Some(format_tags(
            instance.tags.as_ref().unwrap(),
            tags_filter.as_ref().map(|x| x.as_slice()),
        )),
        InstanceDescriptorFields::VirtualizationType => instance.virtualization_type.clone(),
        InstanceDescriptorFields::VpcId => instance.vpc_id.clone(),
    }.unwrap_or_else(|| String::from("-"))
}

/// Format a `HashMap` of `String` -> `Option<String>` into a single line, pretty string.
fn format_tags(tags: &HashMap<String, Option<String>>, tags_filter: Option<&[String]>) -> String {
    let empty = String::from("");
    let mut concat = String::new();

    let mut keys: Vec<_> = if let Some(tags_filter) = tags_filter {
        tags.keys().filter(|&k| tags_filter.contains(k)).collect()
    } else {
        tags.keys().collect()
    };
    keys.sort();
    let mut iter = keys.into_iter();

    if let Some(k) = iter.next() {
        concat.push_str(k);
        concat.push_str("=");
        concat.push_str(tags.get(k).unwrap().as_ref().unwrap_or(&empty));
    };
    for k in iter {
        concat.push_str(", ");
        concat.push_str(k);
        concat.push_str("=");
        concat.push_str(tags.get(k).unwrap().as_ref().unwrap_or(&empty));
    }
    concat
}
