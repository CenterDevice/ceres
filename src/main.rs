extern crate env_logger;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_sts;

use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_ec2::{Ec2, Ec2Client, DescribeInstancesRequest};
use rusoto_sts::{StsClient, StsAssumeRoleSessionCredentialsProvider};

use std::default::Default;

fn main() {
    let _ = env_logger::try_init();

    let base_provider = DefaultCredentialsProvider::new().unwrap();
    let sts = StsClient::new(default_tls_client().unwrap(), base_provider, Region::EuCentral1);

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        "arn:aws:iam::959479900016:role/OrganizationAccountAccessRole".to_string(),
        "default".to_string(),
        None, None, None, None
    );

    let client = Ec2Client::new(default_tls_client().unwrap(), provider, Region::EuCentral1);

    let request = Default::default();
    let instances = client.describe_instances(&request).unwrap();

    println!("{:?}", instances);
}
