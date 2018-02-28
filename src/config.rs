use rusoto_core::Region;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::{self, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub profiles: HashMap<String, Profile>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub provider: Provider
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Provider {
    #[serde(rename = "aws")]
    Aws(AwsProvider)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AwsProvider {
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(serialize_with = "ser_region", deserialize_with = "de_ser_region")]
    pub region: Region,
    pub role_arn: String,
}

fn ser_region<S>(region: &Region, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
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
                .map_err(|e| de::Error::custom(
                    format!("invalid region string '{}'", s)))?;
            Ok(region)
        }
    }

    deserializer.deserialize_string(RegionVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;
    use toml;

    #[test]
    fn serialize_deserialize_round_trip() {
        let aws_provider = AwsProvider {
            access_key_id: String::from("a key id"),
            secret_access_key: String::from("an access key"),
            region: Region::EuCentral1,
            role_arn: String::from("a_role_arn"),
        };
        let prod_profile = Profile { provider: Provider::Aws(aws_provider) };
        let mut profiles = HashMap::new();
        profiles.insert("prod@cd".to_owned(), prod_profile);
        let config = Config { profiles };
        let toml = toml::to_string(&config).unwrap();

        eprintln!("toml = {}", toml);
        assert!(false);

        let re_config: Config = toml::from_str(&toml).unwrap();

        assert_that(&re_config).is_equal_to(&config);
    }
}