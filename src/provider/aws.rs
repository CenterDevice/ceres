use regex::Regex;
use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_ec2::{Ec2, Ec2Client, Instance as Ec2Instance, Tag};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use std::collections::HashMap;
use std::default::Default;

use provider::{Error as ProviderError, ErrorKind as ProviderErrorKind, InstanceDescriptor, DescribeInstances, Result as ProviderResult};

pub struct Aws {
    pub provider_arn: String,
}

impl DescribeInstances for Aws {
    fn describe_instances(&self) -> ProviderResult<Vec<InstanceDescriptor>> {
        list(&self.provider_arn).map_err(
            |e| ProviderError::with_chain(e, ProviderErrorKind::ProviderCallFailed(String::from("describe_instance"))))
    }
}

fn list(provider_arn: &str) -> Result<Vec<InstanceDescriptor>> {
    let role_provider = assume_role(provider_arn)?;
    let default_client = default_tls_client().chain_err(|| ErrorKind::AwsApiError)?;
    let client = Ec2Client::new(default_client, role_provider, Region::EuCentral1);

    let request = Default::default();
    let result = client
        .describe_instances(&request)
        .chain_err(|| ErrorKind::AwsApiError)?;
    let reservations = result.reservations.ok_or_else(|| {
        Error::from_kind(ErrorKind::AwsApiResultError(
            "no reservations found".to_string(),
        ))
    })?;

    let mut instances: Vec<InstanceDescriptor> = Vec::new();
    for r in reservations {
        if let Some(resv_instances) = r.instances {
            for i in resv_instances {
                instances.push(i.into());
            }
        }
    }

    Ok(instances)
    //let (tag_key, tag_value) = (None, None);
    /*
    }
    let (tag_key, tag_value) = match args.value_of("filter") {
        Some(kv) => {
            // cf. https://github.com/rust-lang/rust/issues/23121
            let splits: Vec<_> = kv.split(':').collect();
            match splits.len() {
                1 => (Some(splits[0]), None),
                2 => (Some(splits[0]), Some(splits[1])),
                _ => (None, None),
            }
        }
        None => (None, None),
    };
    */

    /*
    let mut tw = TabWriter::new(vec![]).padding(1);
    writeln!(&mut tw, "  ID\t  Private IP\t  Public IP\t  Tags:Name")
        .chain_err(|| ErrorKind::OutputError)?;
    for resv in reservations {
        let instance_iter = resv.instances
            .as_ref()
            .ok_or_else(|| {
                Error::from_kind(ErrorKind::AwsApiResultError(
                    "no instances found".to_string(),
                ))
            })?
            .iter();
        let empty_tags = Vec::new();
        let instances: Vec<_> = if let Some(tag_key) = tag_key {
            instance_iter
                .filter(|i| {
                    has_tag(
                        i.tags.as_ref().unwrap_or_else(|| &empty_tags),
                        tag_key,
                        tag_value,
                    )
                })
                .collect()
        } else {
            instance_iter.collect()
        };
        let unset = "-".to_string();
        for i in instances {
            let tags = i.tags.as_ref().unwrap_or_else(|| &empty_tags);
            writeln!(
                &mut tw,
                "| {}\t| {}\t| {}\t| {}\t|",
                i.instance_id.as_ref().unwrap_or(&unset),
                i.private_ip_address.as_ref().unwrap_or(&unset),
                i.public_ip_address.as_ref().unwrap_or(&unset),
                get_name_from_tags(tags).unwrap_or(&unset)
            ).chain_err(|| ErrorKind::OutputError)?;
        }
    }
    let out_str = String::from_utf8(tw.into_inner().chain_err(|| ErrorKind::OutputError)?)
        .chain_err(|| ErrorKind::OutputError)?;

    println!("{}", out_str);

    Ok(())
    */
}

impl From<Ec2Instance> for InstanceDescriptor {
    fn from(r: Ec2Instance) -> Self {
        InstanceDescriptor {
            ami_launch_index: r.ami_launch_index,
            architecture: r.architecture,
            //block_device_mappings: r.block_device_mappings,
            client_token: r.client_token,
            ebs_optimized: r.ebs_optimized,
            //elastic_gpu_associations: r.elastic_gpu_associations,
            ena_support: r.ena_support,
            hypervisor: r.hypervisor,
            //iam_instance_profile: r.iam_instance_profile,
            image_id: r.image_id,
            instance_id: r.instance_id,
            instance_lifecycle: r.instance_lifecycle,
            instance_type: r.instance_type,
            kernel_id: r.kernel_id,
            key_name: r.key_name,
            launch_time: r.launch_time,
            //monitoring: r.monitoring,
            //network_interfaces: r.network_interfaces,
            //placement: r.placement,
            platform: r.platform,
            private_dns_name: r.private_dns_name,
            private_ip_address: r.private_ip_address,
            //product_codes: r.product_codes,
            public_dns_name: r.public_dns_name,
            public_ip_address: r.public_ip_address,
            ramdisk_id: r.ramdisk_id,
            root_device_name: r.root_device_name,
            root_device_type: r.root_device_type,
            //security_groups: r.security_groups,
            source_dest_check: r.source_dest_check,
            spot_instance_request_id: r.spot_instance_request_id,
            sriov_net_support: r.sriov_net_support,
            //state: r.state,
            //state_reason: r.state_reason,
            state_transition_reason: r.state_transition_reason,
            subnet_id: r.subnet_id,
            tags: if let Some(tags) = r.tags {Some(vec_tags_to_hashmap(tags))} else { None },
            virtualization_type: r.virtualization_type,
            vpc_id: r.vpc_id,
        }
    }
}

fn vec_tags_to_hashmap(tags: Vec<Tag>) -> HashMap<String, Option<String>> {
    let mut tag_map = HashMap::new();

    for tag in tags {
        if let Some(key) = tag.key {
            tag_map.insert(key, tag.value);
        }
    }

    tag_map
}

fn assume_role(provider_arn: &str) -> Result<StsAssumeRoleSessionCredentialsProvider> {
    let base_provider = DefaultCredentialsProvider::new().chain_err(|| ErrorKind::AwsApiError)?;
    let default_client = default_tls_client().chain_err(|| ErrorKind::AwsApiError)?;
    let sts = StsClient::new(default_client, base_provider, Region::EuCentral1);

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        provider_arn.to_string(),
        "default".to_string(),
        None,
        None,
        None,
        None,
    );

    Ok(provider)
}

fn get_name_from_tags(tags: &[Tag]) -> Option<&String> {
    let tag_name = Some("Name".to_string());
    tags.iter()
        .filter(|tag| tag.key == tag_name)
        .take(1)
        .last()
        .map(|tag| tag.value.as_ref())
        .and_then(|t| t)
}

/// Checks if tags contain a specific tag and optionally with a specific value
/// Tag key and tag value are both matched using regular expressions
///
/// # Examples
///
/// ```
/// # extern crate ceres;
/// # extern crate rusoto_ec2;
/// # use rusoto_ec2::Tag;
/// # use ceres::provider::aws::has_tag;
/// # fn main() {
/// let tags = vec![ Tag{ key: Some("Name".to_string()), value: Some("Example Instance".to_string()) }];
///
/// let res = has_tag(&tags, "Name", None); // true
/// # assert_eq!(res, true);
///
/// let res = has_tag(&tags, "Name", Some("Example Instance")); // true
/// # assert_eq!(res, true);
///
/// let res = has_tag(&tags, "Name", Some("Example")); // true
/// # assert_eq!(res, true);
///
/// let res = has_tag(&tags, "Name", Some("Example$")); // false
/// # assert_eq!(res, false);
///
/// let res = has_tag(&tags, "NoSuchTag", None); // false
/// # assert_eq!(res, false);
/// # }
/// ```
/// TODO: Refactor: Move RE compilation out
pub fn has_tag(tags: &[Tag], key: &str, value: Option<&str>) -> bool {
    let key_re = Regex::new(key).unwrap();
    let pred: Box<Fn(&Tag) -> bool> = match (key, value) {
        (_, None) => Box::new(|tag: &Tag| key_re.is_match(tag.key.as_ref().unwrap())),
        (_, Some(v)) => {
            let value_re = Regex::new(v).unwrap();
            Box::new(move |tag: &Tag| {
                key_re.is_match(tag.key.as_ref().unwrap())
                    && value_re.is_match(tag.value.as_ref().unwrap())
            })
        }
    };
    tags.iter().any(|tag| pred(tag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_ec2::Tag;
    use spectral::prelude::*;

    #[test]
    fn get_name_from_tags_okay() {
        let tags = vec![
            Tag {
                key: Some("Name".to_string()),
                value: Some("Example Instance".to_string()),
            },
        ];

        let result = get_name_from_tags(&tags);

        asserting(&"name tag included")
            .that(&result)
            .is_some()
            .is_equal_to(&"Example Instance".to_string());
    }

    #[test]
    fn get_name_from_tags_fails() {
        let tags = vec![
            Tag {
                key: Some("NoName".to_string()),
                value: Some("Example Instance".to_string()),
            },
        ];

        let result = get_name_from_tags(&tags);

        asserting(&"name tag not included").that(&result).is_none();
    }
}


error_chain! {
errors {
AwsApiError {
description("Call to AWS API failed.")
}
AwsApiResultError(reason: String) {
description("Unexpected result.")
display("Unexpected result because {}.", reason)
}
RegExError {
description("RegEx failed.")
}
SubcommandError {
description("Invalid Subcommand specified.")
}
OutputError {
description("Failed to write output.")
}
}
}
