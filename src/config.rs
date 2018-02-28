use rusoto_core::Region;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use std::collections::HashMap;
use std::fs::File;
use std::fmt;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use toml;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub profiles: HashMap<String, Profile>
}

impl Config {
    pub fn from_file<T: AsRef<Path>>(file_path: T) -> Result<Config> {
        let mut file = File::open(file_path)?;
        let content = Config::read_to_string(&mut file)?;

        Config::parse_toml(&content)
    }

    fn read_to_string(file: &mut File) -> Result<String> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    fn parse_toml(content: &str) -> Result<Config> {
        let config: Config = toml::from_str(content)?;

        Ok(config)
    }
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


error_chain! {
    errors {
    }
    foreign_links {
        CouldNotRead(::std::io::Error);
        CouldNotParse(::toml::de::Error);
    }
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

        let re_config: Config = toml::from_str(&toml).unwrap();

        assert_that(&re_config).is_equal_to(&config);
    }

    #[test]
    fn load_from_file() {
        let config = Config::from_file("examples/ceres.conf").unwrap();

        assert_that(&config.profiles).contains_key(String::from("default"));
        let default_profile = config.profiles.get("default").unwrap();

        let &Provider::Aws(ref aws) = &default_profile.provider;
        assert_that(&aws.access_key_id).is_equal_to("a key id".to_owned());
        assert_that(&aws.secret_access_key).is_equal_to("an access key".to_owned());
        assert_that(&aws.region).is_equal_to(Region::EuCentral1);
        assert_that(&aws.role_arn).is_equal_to("a_role_arn".to_owned());
    }
}