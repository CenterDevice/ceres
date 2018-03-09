use rusoto_core::{default_tls_client, Region};
use rusoto_credential::StaticProvider;
use rusoto_ec2::{self as ec2, DescribeInstancesRequest, Ec2, TerminateInstancesRequest, TerminateInstancesError};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::str::FromStr;

use provider::{Error as ProviderError, ErrorKind as ProviderErrorKind, InstanceDescriptor, InstanceId, DescribeInstance, DescribeInstances, Result as ProviderResult, StateChange, TerminateInstances};

const EMPTY: &str = "-";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Aws {
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(serialize_with = "ser_region", deserialize_with = "de_ser_region")]
    pub region: Region,
    pub role_arn: String,
}

fn ser_region<S>(region: &Region, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(region.name())
}

fn de_ser_region<'de, D>(deserializer: D) -> ::std::result::Result<Region, D::Error> where D: Deserializer<'de> {
    struct RegionVisitor;

    impl<'a> Visitor<'a> for RegionVisitor {
        type Value = Region;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("valid AWS region string")
        }

        fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E> where E: de::Error {
            let region = Region::from_str(s)
                .map_err(|_| de::Error::custom(
                    format!("invalid region string '{}'", s)))?;
            Ok(region)
        }
    }

    deserializer.deserialize_string(RegionVisitor)
}

impl DescribeInstances for Aws {
    fn describe_instances(&self) -> ProviderResult<Vec<InstanceDescriptor>> {
        list(self).map_err(
            |e| ProviderError::with_chain(e, ProviderErrorKind::ProviderCallFailed(String::from("describe_instance"))))
    }
}

fn list(aws: &Aws) -> Result<Vec<InstanceDescriptor>> {
    let credentials_provider = assume_role(aws)?;
    let default_client = default_tls_client().chain_err(|| ErrorKind::AwsApiError)?;
    let client = ec2::Ec2Client::new(default_client, credentials_provider, aws.region.clone());

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
}

impl DescribeInstance for Aws {
    fn describe_instance(&self, instance_id: &str) -> ProviderResult<InstanceDescriptor> {
        describe(self, instance_id).map_err(
            |e| ProviderError::with_chain(e, ProviderErrorKind::ProviderCallFailed(String::from("describe_instance"))))
    }
}

fn describe(aws: &Aws, instance_id: &str) -> Result<InstanceDescriptor> {
    let credentials_provider = assume_role(aws)?;
    let default_client = default_tls_client().chain_err(|| ErrorKind::AwsApiError)?;
    let client = ec2::Ec2Client::new(default_client, credentials_provider, aws.region.clone());

    let request = DescribeInstancesRequest {
        dry_run: Some(false),
        filters: None,
        instance_ids: Some(vec![instance_id.to_string()]),
        max_results: None,
        next_token: None,
    };
    let result = client
        .describe_instances(&request)
        .chain_err(|| ErrorKind::AwsApiError)?;
    let mut reservations = result.reservations.ok_or_else(|| {
        Error::from_kind(ErrorKind::AwsApiResultError(
            "no reservations found".to_string(),
        ))
    })?;
    let first_reservation = reservations.pop().ok_or_else(|| {
        Error::from_kind(ErrorKind::AwsApiResultError(
            "no reservations received".to_string(),
        ))
    })?;
    let mut instances = first_reservation.instances.ok_or_else(|| {
        Error::from_kind(ErrorKind::AwsApiResultError(
            "no instances in reservation found".to_string(),
        ))
    })?;
    let instance = instances.pop().ok_or_else(|| {
        Error::from_kind(ErrorKind::AwsApiResultError(
            "no instances received".to_string(),
        ))
    })?;

    Ok(instance.into())
}



impl From<ec2::Instance> for InstanceDescriptor {
    fn from(r: ec2::Instance) -> Self {
        InstanceDescriptor {
            ami_launch_index: r.ami_launch_index,
            architecture: r.architecture,
            block_device_mappings: r.block_device_mappings
                .map(|bdms| bdms
                    .iter()
                    .map(block_device_mapping_to_string).collect()
                ),
            client_token: r.client_token,
            ebs_optimized: r.ebs_optimized,
            //elastic_gpu_associations: r.elastic_gpu_associations,
            ena_support: r.ena_support,
            hypervisor: r.hypervisor,
            iam_instance_profile: r.iam_instance_profile.as_ref().map(iam_instance_profile_to_string),
            image_id: r.image_id,
            instance_id: r.instance_id,
            instance_lifecycle: r.instance_lifecycle,
            instance_type: r.instance_type,
            kernel_id: r.kernel_id,
            key_name: r.key_name,
            launch_time: r.launch_time,
            monitoring: r.monitoring.as_ref().map(monitoring_to_string),
            //network_interfaces: r.network_interfaces,
            placement: r.placement.as_ref().map(placement_to_string),
            platform: r.platform,
            private_dns_name: r.private_dns_name,
            private_ip_address: r.private_ip_address,
            //product_codes: r.product_codes,
            public_dns_name: r.public_dns_name,
            public_ip_address: r.public_ip_address,
            ramdisk_id: r.ramdisk_id,
            root_device_name: r.root_device_name,
            root_device_type: r.root_device_type,
            security_groups: r.security_groups
                .map(|bdms| bdms
                    .iter()
                    .map(group_identifier_to_string).collect()
                ),
            source_dest_check: r.source_dest_check,
            spot_instance_request_id: r.spot_instance_request_id,
            sriov_net_support: r.sriov_net_support,
            state: r.state.as_ref().map(instance_state_to_string),
            state_reason: r.state_reason.as_ref().map(state_reason_to_string),
            state_transition_reason: r.state_transition_reason,
            subnet_id: r.subnet_id,
            tags: if let Some(tags) = r.tags { Some(vec_tags_to_hashmap(tags)) } else { None },
            virtualization_type: r.virtualization_type,
            vpc_id: r.vpc_id,
        }
    }
}

fn block_device_mapping_to_string(bdm: &ec2::InstanceBlockDeviceMapping) -> String {
    let empty = String::from(EMPTY);
    format!("dev={}, id={}",
            bdm.device_name.as_ref().unwrap_or(&empty),
            bdm.ebs.as_ref().unwrap().volume_id.as_ref().unwrap_or(&empty),
    )
}

fn iam_instance_profile_to_string(iip: &ec2::IamInstanceProfile) -> String {
    let empty = String::from(EMPTY);
    format!("id={}, arn={}",
            iip.id.as_ref().unwrap_or(&empty),
            iip.arn.as_ref().unwrap_or(&empty),
    )
}

fn monitoring_to_string(m: &ec2::Monitoring) -> String {
    let empty = String::from(EMPTY);
    format!("{}", m.state.as_ref().unwrap_or(&empty))
}

fn placement_to_string(p: &ec2::Placement) -> String {
    let empty = String::from(EMPTY);
    format!("affinity={}, AZ={}, group={}, host={}, tenancy={}",
            p.affinity.as_ref().unwrap_or(&empty),
            p.availability_zone.as_ref().unwrap_or(&empty),
            p.group_name.as_ref().unwrap_or(&empty),
            p.host_id.as_ref().unwrap_or(&empty),
            p.tenancy.as_ref().unwrap_or(&empty),
    )
}

fn group_identifier_to_string(sg: &ec2::GroupIdentifier) -> String {
    let empty = String::from(EMPTY);
    format!("id={}, name={}",
            sg.group_id.as_ref().unwrap_or(&empty),
            sg.group_name.as_ref().unwrap_or(&empty),
    )
}

fn instance_state_to_string(state: &ec2::InstanceState) -> String {
    let empty = String::from(EMPTY);
    format!("{}", state.name.as_ref().unwrap_or(&empty))
}

fn state_reason_to_string(reason: &ec2::StateReason) -> String {
    let empty = String::from(EMPTY);
    format!("'{}'", reason.message.as_ref().unwrap_or(&empty))
}

fn vec_tags_to_hashmap(tags: Vec<ec2::Tag>) -> HashMap<String, Option<String>> {
    let mut tag_map = HashMap::new();
    for tag in tags {
        if let Some(key) = tag.key {
            tag_map.insert(key, tag.value);
        }
    }

    tag_map
}

fn assume_role(aws: &Aws) -> Result<StsAssumeRoleSessionCredentialsProvider> {
    //let base_provider = DefaultCredentialsProvider::new().chain_err(|| ErrorKind::AwsApiError)?;
    let base_provider = StaticProvider::new(
        aws.access_key_id.clone(),
        aws.secret_access_key.clone(),
        None,
        None,
    );
    let default_client = default_tls_client().chain_err(|| ErrorKind::AwsApiError)?;
    let sts = StsClient::new(default_client, base_provider, aws.region.clone());

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        aws.role_arn.clone(),
        "default".to_string(),
        None,
        None,
        None,
        None,
    );

    Ok(provider)
}

impl TerminateInstances for Aws {
    fn terminate_instances(&self, dry: bool, instance_ids: &[InstanceId]) -> ProviderResult<Vec<StateChange>> {
        destroy(self, dry, instance_ids).map_err(
            |e| ProviderError::with_chain(e, ProviderErrorKind::ProviderCallFailed(String::from("terminate_instances"))))
    }
}

fn destroy(aws: &Aws, dry: bool, instance_ids: &[InstanceId]) -> Result<Vec<StateChange>> {
    let credentials_provider = assume_role(aws)?;
    let default_client = default_tls_client().chain_err(|| ErrorKind::AwsApiError)?;
    let client = ec2::Ec2Client::new(default_client, credentials_provider, aws.region.clone());

    let request = TerminateInstancesRequest {
        dry_run: Some(dry),
        instance_ids: instance_ids.iter().map(|x| x.to_owned()).collect::<Vec<_>>(),
    };
    // If run in dry mode, AWS returns an error of type DryRunOperation
    // cf. https://docs.rs/rusoto_ec2/0.31.0/rusoto_ec2/struct.TerminateInstancesRequest.html#structfield.dry_run
    let result = match client.terminate_instances(&request) {
        Err(TerminateInstancesError::Unknown(ref s)) if s.contains("DryRunOperation") => return Ok(create_dry_run_results(instance_ids)),
        Err(TerminateInstancesError::Unknown(ref s)) if s.contains("UnauthorizedOperation") => return Err(Error::from_kind(ErrorKind::AwsApiResultError("termination is not authorized".to_string()))),
        result => result
    }.chain_err(|| ErrorKind::AwsApiError)?;
    let terminating_instances = result.terminating_instances.ok_or_else(|| {
        Error::from_kind(ErrorKind::AwsApiResultError(
            "termination failed".to_string(),
        ))
    })?;

    let state_changes: Vec<StateChange> = terminating_instances
        .into_iter()
        .map(|x| x.into())
        .collect();

    Ok(state_changes)
}

fn create_dry_run_results(instance_ids: &[InstanceId]) -> Vec<StateChange> {
    instance_ids
        .iter()
        .map(|i| StateChange {
            instance_id: i.to_owned(),
            previous_state: "- n/a -".to_owned(),
            current_state: "- n/a -".to_owned(),
        })
        .collect::<Vec<_>>()
}

impl From<ec2::InstanceStateChange> for StateChange {
    fn from(x: ec2::InstanceStateChange) -> Self {
        StateChange {
            instance_id: x.instance_id.unwrap_or_else(|| String::from("- n/a -")),
            // TODO: Fix me!
            current_state: x.current_state
                            .map(|x| x.name.unwrap_or_else(|| String::from("- n/a -")))
                            .unwrap_or_else(|| String::from("- n/a -")),
            previous_state: x.previous_state
                            .map(|x| x.name.unwrap_or_else(|| String::from("- n/a -")))
                            .unwrap_or_else(|| String::from("- n/a -")),
        }
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
