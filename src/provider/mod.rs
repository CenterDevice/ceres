use std::collections::HashMap;
use std::str::FromStr;

pub mod aws;

pub trait DescribeInstances {
    fn describe_instances(&self) -> Result<Vec<InstanceDescriptor>>;
}

pub enum InstanceDescriptorFields {
    BlockDeviceMapping,
    Hypervisor,
    IamInstanceProfile,
    ImageId,
    InstanceId,
    InstanceType,
    LaunchTime,
    Monitoring,
    Placement,
    PrivateDnsName,
    PrivateIpAddress,
    PublicDnsName,
    PublicIpAddress,
    RootDeviceName,
    RootDeviceType,
    SecurityGroups,
    State,
    StateReason,
    Tags(Option<Vec<String>>),
    VirtualizationType,
    VpcId,
}

impl FromStr for InstanceDescriptorFields {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "BlockDeviceMapping" => Ok(InstanceDescriptorFields::BlockDeviceMapping),
            "Hypervisor" => Ok(InstanceDescriptorFields::Hypervisor),
            "IamInstanceProfile" => Ok(InstanceDescriptorFields::IamInstanceProfile),
            "ImageId" => Ok(InstanceDescriptorFields::ImageId),
            "InstanceId" => Ok(InstanceDescriptorFields::InstanceId),
            "InstanceType" => Ok(InstanceDescriptorFields::InstanceType),
            "LaunchTime" => Ok(InstanceDescriptorFields::LaunchTime),
            "Monitoring" => Ok(InstanceDescriptorFields::Monitoring),
            "Placement" => Ok(InstanceDescriptorFields::Placement),
            "PrivateDnsName" => Ok(InstanceDescriptorFields::PrivateDnsName),
            "PrivateIpAddress" => Ok(InstanceDescriptorFields::PrivateIpAddress),
            "PublicDnsName" => Ok(InstanceDescriptorFields::PublicDnsName),
            "PublicIpAddress" => Ok(InstanceDescriptorFields::PublicIpAddress),
            "RootDeviceName" => Ok(InstanceDescriptorFields::RootDeviceName),
            "RootDeviceType" => Ok(InstanceDescriptorFields::RootDeviceType),
            "SecurityGroups" => Ok(InstanceDescriptorFields::SecurityGroups),
            "State" => Ok(InstanceDescriptorFields::State),
            "StateReason" => Ok(InstanceDescriptorFields::StateReason),
            s if s.starts_with("Tags") => {
                let tags_filter = extract_tags_filter(s);
                Ok(InstanceDescriptorFields::Tags(tags_filter))
            }
            "VirtualizationType" => Ok(InstanceDescriptorFields::VirtualizationType),
            "VpcId" => Ok(InstanceDescriptorFields::VpcId),
            _ => Err(Error::from_kind(ErrorKind::InstanceDescriptorFieldsParsingFailed(s.to_owned())))
        }
    }
}

fn extract_tags_filter(tags_str: &str) -> Option<Vec<String>> {
    if tags_str.len() < 5 { return None };
    let tags = &tags_str[5..]; // Safe because we call this function only when the prefix 'Tags:' has been seen
    let tags_filter: Vec<_> = tags.split(':').map(String::from).collect();

    Some(tags_filter)
}

#[derive(Serialize)]
pub struct InstanceDescriptor {
    pub ami_launch_index: Option<i64>,
    pub architecture: Option<String>,
    pub block_device_mappings: Vec<String>,
    pub client_token: Option<String>,
    pub ebs_optimized: Option<bool>,
    // Won't convert this
    //pub elastic_gpu_associations: Option<Vec<ElasticGpuAssociation>>,
    pub ena_support: Option<bool>,
    pub hypervisor: Option<String>,
    pub iam_instance_profile: Option<String>,
    pub image_id: Option<String>,
    pub instance_id: Option<String>,
    pub instance_lifecycle: Option<String>,
    pub instance_type: Option<String>,
    pub kernel_id: Option<String>,
    pub key_name: Option<String>,
    pub launch_time: Option<String>,
    pub monitoring: Option<String>,
    // TODO: network_interfaces contains a lot of useful information but it's a data structure rabbit hole
    //pub network_interfaces: Option<Vec<InstanceNetworkInterface>>,
    pub placement: Option<String>,
    pub platform: Option<String>,
    pub private_dns_name: Option<String>,
    pub private_ip_address: Option<String>,
    // Won't convert this
    //pub product_codes: Option<Vec<ProductCode>>,
    pub public_dns_name: Option<String>,
    pub public_ip_address: Option<String>,
    pub ramdisk_id: Option<String>,
    pub root_device_name: Option<String>,
    pub root_device_type: Option<String>,
    pub security_groups: Vec<String>,
    pub source_dest_check: Option<bool>,
    pub spot_instance_request_id: Option<String>,
    pub sriov_net_support: Option<String>,
    pub state: Option<String>,
    pub state_reason: Option<String>,
    pub state_transition_reason: Option<String>,
    pub subnet_id: Option<String>,
    pub tags: Option<HashMap<String, Option<String>>>,
    pub virtualization_type: Option<String>,
    pub vpc_id: Option<String>,
}

impl Default for InstanceDescriptor {
    fn default() -> Self {
        InstanceDescriptor {
            ami_launch_index: None,
            architecture: None,
            block_device_mappings: Vec::new(),
            client_token: None,
            ebs_optimized: None,
            ena_support: None,
            hypervisor: None,
            iam_instance_profile: None,
            image_id: None,
            instance_id: None,
            instance_lifecycle: None,
            instance_type: None,
            kernel_id: None,
            key_name: None,
            launch_time: None,
            monitoring: None,
            placement: None,
            platform: None,
            private_dns_name: None,
            private_ip_address: None,
            public_dns_name: None,
            public_ip_address: None,
            ramdisk_id: None,
            root_device_name: None,
            root_device_type: None,
            security_groups: Vec::new(),
            source_dest_check: None,
            spot_instance_request_id: None,
            sriov_net_support: None,
            state: None,
            state_reason: None,
            state_transition_reason: None,
            subnet_id: None,
            tags: None,
            virtualization_type: None,
            vpc_id: None,

        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;

    #[test]
    fn extract_tags_filter_empty() {
        let tag_str = "Tag";

        let res = extract_tags_filter(&tag_str);

        assert_that(&res).is_none();
    }

    #[test]
    fn extract_tags_filter_one_tag() {
        let tag_str = "Tags:Name";

        let res = extract_tags_filter(&tag_str);

        assert_that(&res).is_some().is_equal_to(vec!["Name".to_owned()]);
    }

    #[test]
    fn extract_tags_filter_two_tag() {
        let tag_str = "Tags:Name:SomeOtherTag";

        let res = extract_tags_filter(&tag_str);

        assert_that(&res).is_some().is_equal_to(vec!["Name".to_owned(), "SomeOtherTag".to_owned()]);
    }
}