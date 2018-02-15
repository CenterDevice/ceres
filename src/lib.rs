extern crate regex;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_sts;
extern crate tabwriter;

use regex::Regex;
use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_ec2::{Ec2, Ec2Client, Tag};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use std::default::Default;
use std::io::Write;
use tabwriter::TabWriter;

pub fn noop() -> Result<(), ()> {
    Ok(())
}

pub fn instances_list(provider_arn: &str, tag_key: Option<&str>, tag_value: Option<&str>) {
    let base_provider = DefaultCredentialsProvider::new().unwrap();
    let sts = StsClient::new(
        default_tls_client().unwrap(),
        base_provider,
        Region::EuCentral1,
    );

    let provider = StsAssumeRoleSessionCredentialsProvider::new(
        sts,
        provider_arn.to_string(),
        "default".to_string(),
        None,
        None,
        None,
        None,
    );

    let client = Ec2Client::new(default_tls_client().unwrap(), provider, Region::EuCentral1);

    let request = Default::default();
    let result = client.describe_instances(&request).unwrap();

    let mut tw = TabWriter::new(vec![]).padding(1);
    writeln!(&mut tw, "  ID\t  Private IP\t  Public IP\t  Tags:Name").unwrap();
    for resv in result.reservations.unwrap() {
        //writeln!(&mut tw, "Reservation ID: '{}'", resv.reservation_id.as_ref().unwrap()).unwrap();
        let instances: Vec<_> = if let Some(tag_key) = tag_key {
            resv.instances
                .as_ref()
                .unwrap()
                .iter()
                .filter(|i| has_tag(i.tags.as_ref().unwrap(), tag_key, tag_value))
                .collect()
        } else {
            resv.instances.as_ref().unwrap().iter().collect()
        };
        for i in instances {
            writeln!(
                &mut tw,
                "| {}\t| {}\t| {}\t| {}\t|",
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
/// # use ceres::has_tag;
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

    #[test]
    fn get_name_from_tags_okay() {
        let tags = vec![
            Tag {
                key: Some("Name".to_string()),
                value: Some("Example Instance".to_string()),
            },
        ];

        let result = get_name_from_tags(&tags);

        assert_eq!(result, Some(&"Example Instance".to_string()));
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

        assert_eq!(result, None);
    }
}
