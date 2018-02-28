use std::collections::HashMap;
use std::str::FromStr;

pub mod aws;

pub trait DescribeInstances {
    fn describe_instances(&self) -> Result<Vec<InstanceDescriptor>>;
}

pub enum InstanceDescriptorFields {
    Hypervisor,
    InstanceId,
    InstanceType,
    LaunchTime,
    PrivateDnsName,
    PrivateIpAddress,
    PublicDnsName,
    PublicIpAddress,
    RootDeviceName,
    RootDeviceType,
    Tags,
}

impl FromStr for InstanceDescriptorFields {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "Hypervisor" => Ok(InstanceDescriptorFields::Hypervisor),
            "InstanceId" => Ok(InstanceDescriptorFields::InstanceId),
            "InstanceType" => Ok(InstanceDescriptorFields::InstanceType),
            "LaunchTime" => Ok(InstanceDescriptorFields::LaunchTime),
            "PrivateDnsName" => Ok(InstanceDescriptorFields::PrivateDnsName),
            "PrivateIpAddress" => Ok(InstanceDescriptorFields::PrivateIpAddress),
            "PublicDnsName" => Ok(InstanceDescriptorFields::PublicDnsName),
            "PublicIpAddress" => Ok(InstanceDescriptorFields::PublicIpAddress),
            "RootDeviceName" => Ok(InstanceDescriptorFields::RootDeviceName),
            "RootDeviceType" => Ok(InstanceDescriptorFields::RootDeviceType),
            "Tags" => Ok(InstanceDescriptorFields::Tags),
            _ => Err(Error::from_kind(ErrorKind::InstanceDescriptorFieldsParsingFailed(s.to_owned())))
        }
    }
}

pub struct InstanceDescriptor {
    pub ami_launch_index: Option<i64>,
    pub architecture: Option<String>,
    //pub block_device_mappings: Option<Vec<InstanceBlockDeviceMapping>>,
    pub client_token: Option<String>,
    pub ebs_optimized: Option<bool>,
    //pub elastic_gpu_associations: Option<Vec<ElasticGpuAssociation>>,
    pub ena_support: Option<bool>,
    pub hypervisor: Option<String>,
    //pub iam_instance_profile: Option<IamInstanceProfile>,
    pub image_id: Option<String>,
    pub instance_id: Option<String>,
    pub instance_lifecycle: Option<String>,
    pub instance_type: Option<String>,
    pub kernel_id: Option<String>,
    pub key_name: Option<String>,
    pub launch_time: Option<String>,
    //pub monitoring: Option<Monitoring>,
    //pub network_interfaces: Option<Vec<InstanceNetworkInterface>>,
    //pub placement: Option<Placement>,
    pub platform: Option<String>,
    pub private_dns_name: Option<String>,
    pub private_ip_address: Option<String>,
    //pub product_codes: Option<Vec<ProductCode>>,
    pub public_dns_name: Option<String>,
    pub public_ip_address: Option<String>,
    pub ramdisk_id: Option<String>,
    pub root_device_name: Option<String>,
    pub root_device_type: Option<String>,
    //pub security_groups: Option<Vec<GroupIdentifier>>,
    pub source_dest_check: Option<bool>,
    pub spot_instance_request_id: Option<String>,
    pub sriov_net_support: Option<String>,
    //pub state: Option<InstanceState>,
    //pub state_reason: Option<StateReason>,
    pub state_transition_reason: Option<String>,
    pub subnet_id: Option<String>,
    pub tags: Option<HashMap<String, Option<String>>>,
    pub virtualization_type: Option<String>,
    pub vpc_id: Option<String>,
}

error_chain! {
    errors {
        ProviderCallFailed(call: String) {
            description("API call to provider failed.")
            display("API call '{}' to provider failed.", call)
        }
        InstanceDescriptorFieldsParsingFailed(s: String) {
            description("Failed to parse InstanceDescriptorFields from String.")
            display("Failed to parse InstanceDescriptorFields from String '{}'.", s)
        }
    }
}