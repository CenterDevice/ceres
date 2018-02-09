extern crate env_logger;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_sts;
extern crate tabwriter;

use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_ec2::{Ec2, Ec2Client, DescribeInstancesRequest, Tag};
use rusoto_sts::{StsClient, StsAssumeRoleSessionCredentialsProvider};
use std::default::Default;
use std::io::{self, Write};
use tabwriter::TabWriter;


fn main() {
    let _ = env_logger::try_init();

    let base_provider = DefaultCredentialsProvider::new().unwrap();
    let sts = StsClient::new(default_tls_client().unwrap(), base_provider, Region::EuCentral1);

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        //"arn:aws:iam::959479900016:role/OrganizationAccountAccessRole".to_string(),
        "arn:aws:iam::737288212407:role/OrganizationAccountAccessRole".to_string(),
        "default".to_string(),
        None, None, None, None
    );

    let client = Ec2Client::new(default_tls_client().unwrap(), provider, Region::EuCentral1);

    let request = Default::default();
    let result = client.describe_instances(&request).unwrap();

    let mut tw = TabWriter::new(vec![]).padding(1);
    writeln!(&mut tw, "  ID\t  Private IP\t  Public IP\t  Tags:Name").unwrap();
    for resv in result.reservations.unwrap() {
        //writeln!(&mut tw, "Reservation ID: '{}'", resv.reservation_id.as_ref().unwrap()).unwrap();
        for i in resv.instances.as_ref().unwrap() {
            writeln!(&mut tw, "| {}\t| {}\t| {}\t| {}\t|",
                     i.instance_id.as_ref().unwrap(),
                     i.private_ip_address.as_ref().unwrap(),
                     i.public_ip_address.as_ref().unwrap(),
                     get_name_from_tags(i.tags.as_ref().unwrap()).unwrap()
                     ).unwrap();
        }
    }
    let out_str = String::from_utf8(tw.into_inner().unwrap()).unwrap();

    println!("{}", out_str);
}

fn get_name_from_tags(tags: &Vec<Tag>) -> Option<&String> {
    let tag_name = Some("Name".to_string());
    tags.iter()
        .filter(|tag| tag.key == tag_name)
        .take(1)
        .last()
        .map(|tag| tag.value.as_ref())
        .and_then(|t| t)
}
