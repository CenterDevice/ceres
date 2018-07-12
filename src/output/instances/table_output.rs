use prettytable::cell::Cell;
use prettytable::format;
use prettytable::row::Row;
use prettytable::Table;
use std::collections::HashMap;
use std::io::Write;
use utils::command::ExitStatus;

use output::instances::*;
use provider::{InstanceDescriptor, InstanceDescriptorFields, StateChange};
use utils::command::CommandResult;

pub struct TableOutputInstances {
    pub fields: Vec<InstanceDescriptorFields>,
}

impl Default for TableOutputInstances {
    fn default() -> Self {
        TableOutputInstances {
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

impl OutputInstances for TableOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(
            self.fields
                .iter()
                .map(|f| Cell::new(header_for_field(f)))
                .collect::<Vec<_>>(),
        ));

        // We have to create / allocate the Strings first since `Table` only accepts `&str` and some
        // `InstanceDescriptorFields` need to allocate representations first, e.g., `InstanceDescriptorFields::Tags`
        let mut rows = Vec::new();
        for instance in instances {
            let row = self
                .fields
                .iter()
                .map(|f| value_for_field(f, instance))
                .collect::<Vec<_>>();
            rows.push(row);
        }
        for r in rows {
            table.add_row(Row::new(
                r.iter().map(|cell| Cell::new(cell)).collect::<Vec<_>>(),
            ));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

fn header_for_field(field: &InstanceDescriptorFields) -> &str {
    match *field {
        InstanceDescriptorFields::BlockDeviceMappings => "Block Device Mappings",
        InstanceDescriptorFields::Hypervisor => "Hypervisor",
        InstanceDescriptorFields::IamInstanceProfile => "Iam Instance Profile",
        InstanceDescriptorFields::ImageId => "Image Id",
        InstanceDescriptorFields::InstanceId => "Instance Id",
        InstanceDescriptorFields::InstanceType => "Instance Type",
        InstanceDescriptorFields::LaunchTime => "Launch Time",
        InstanceDescriptorFields::Monitoring => "Monitoring",
        InstanceDescriptorFields::Placement => "Placement",
        InstanceDescriptorFields::PrivateDnsName => "Private DNS Name",
        InstanceDescriptorFields::PrivateIpAddress => "Private IP Address",
        InstanceDescriptorFields::PublicDnsName => "Public DNS Name",
        InstanceDescriptorFields::PublicIpAddress => "Public IP Address",
        InstanceDescriptorFields::RootDeviceName => "Root Device Name",
        InstanceDescriptorFields::RootDeviceType => "Root Device Type",
        InstanceDescriptorFields::SecurityGroups => "Security Groups",
        InstanceDescriptorFields::State => "State",
        InstanceDescriptorFields::StateReason => "State Reason",
        InstanceDescriptorFields::Tags(_) => "Tags",
        InstanceDescriptorFields::VirtualizationType => "Virtualization Type",
        InstanceDescriptorFields::VpcId => "Vpc Id",
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
        InstanceDescriptorFields::Tags(ref tags_filter) => {
            if let Some(ref tags) = instance.tags.as_ref() {
                Some(format_tags(
                    tags,
                    tags_filter.as_ref().map(|x| x.as_slice()),
                ))
            } else {
                None
            }
        }
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

pub struct TableOutputStatusChanges;

impl OutputStateChanges for TableOutputStatusChanges {
    fn output<T: Write>(&self, writer: &mut T, state_changes: &[StateChange]) -> Result<()> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(
            ["Instance Id", "Previous State", "Current State"]
                .iter()
                .map(|x| Cell::new(x))
                .collect::<Vec<_>>(),
        ));

        for change in state_changes {
            table.add_row(Row::new(vec![
                Cell::new(&change.instance_id),
                Cell::new(&change.previous_state),
                Cell::new(&change.current_state),
            ]));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct TableOutputCommandResults {
    pub show_all: bool,
}

impl OutputCommandResults for TableOutputCommandResults {
    fn output<T: Write>(&self, writer: &mut T, results: &[CommandResult]) -> Result<()> {
        let results = TableOutputCommandResults::filter_results(results, self.show_all);
        if results.is_empty() {
            return Ok(());
        };

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(
            ["Command Id", "Exit Status", "Log File"]
                .iter()
                .map(|x| Cell::new(x))
                .collect::<Vec<_>>(),
        ));

        for r in results {
            table.add_row(Row::new(vec![
                Cell::new(&r.id),
                Cell::new(&format!("{:?}", r.exit_status)),
                Cell::new(&format!("{}", r.log.to_str().unwrap_or_else(|| "- n/a -"))),
            ]));
        }

        table.print(writer).chain_err(|| ErrorKind::OutputFailed)
    }
}

impl TableOutputCommandResults {
    fn filter_results<'a>(results: &'a [CommandResult], show_all: bool) -> Vec<&CommandResult> {
        if show_all {
            info!("Outputting all result.");
            results.iter().filter(|_| true).collect()
        } else {
            info!("Outputting only failed result.");
            results
                .iter()
                .filter(|x| x.exit_status != ExitStatus::Exited(0))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn format_tags_empty() {
        let tags = HashMap::new();

        let res = format_tags(&tags, None);

        let expected = String::from("");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_one_kv() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));

        let res = format_tags(&tags, None);

        let expected = String::from("key1=value1");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_multiple_kv() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));
        tags.insert("key2".to_owned(), None);
        tags.insert("key3".to_owned(), Some("value3".to_owned()));

        let res = format_tags(&tags, None);

        let expected = String::from("key1=value1, key2=, key3=value3");
        assert_that(&res).is_equal_to(&expected);
    }

    #[test]
    fn format_tags_multiple_kv_with_filter() {
        let mut tags = HashMap::new();
        tags.insert("key1".to_owned(), Some("value1".to_owned()));
        tags.insert("key2".to_owned(), None);
        tags.insert("key3".to_owned(), Some("value3".to_owned()));
        let filter: &[String] = &["key1".to_owned(), "key3".to_owned()];
        let tags_filter = Some(filter);

        let res = format_tags(&tags, tags_filter);

        let expected = String::from("key1=value1, key3=value3");
        assert_that(&res).is_equal_to(&expected);
    }
}
