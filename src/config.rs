use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

use provider;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub default_profile: String,
    pub profiles: HashMap<String, Profile>
}

impl Config {
    pub fn from_file<T: AsRef<Path>>(file_path: T) -> Result<Config> {
        let mut file = File::open(file_path)?;
        let content = read_to_string(&mut file)?;

        parse_toml(&content)
    }

    pub fn get_profile(&self, profile_name: &str) -> Result<&Profile> {
        let profile = self.profiles.get(profile_name)
            .ok_or_else(|| ErrorKind::NoSuchProfile(profile_name.to_owned()))?;

        Ok(&profile)
    }

    pub fn get_default_profile(&self) -> Result<&Profile> {
        self.get_profile(&self.default_profile)
    }
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub ssh_user: Option<String>,
    pub provider: Provider
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Provider {
    #[serde(rename = "aws")]
    Aws(provider::aws::Aws)
}

error_chain! {
    errors {
        NoSuchProfile(profile: String) {
            description("No such profile")
            display("No such profile '{}'", profile)
        }
    }
    foreign_links {
        CouldNotRead(::std::io::Error);
        CouldNotParse(::toml::de::Error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rusoto_core::Region;
    use spectral::prelude::*;
    use toml;

    #[test]
    fn serialize_deserialize_round_trip() {
        let aws_provider = provider::aws::Aws {
            access_key_id: String::from("a key id"),
            secret_access_key: String::from("an access key"),
            region: Region::EuCentral1,
            role_arn: String::from("a_role_arn"),
        };
        let prod_profile = Profile { ssh_user: Some("a_user".to_owned()), provider: Provider::Aws(aws_provider) };
        let mut profiles = HashMap::new();
        profiles.insert("prod@cd".to_owned(), prod_profile);
        let config = Config { default_profile: "prod@cd".to_owned(), profiles };
        let toml = toml::to_string(&config).unwrap();

        let re_config: Config = toml::from_str(&toml).unwrap();

        assert_that(&re_config).is_equal_to(&config);
    }

    #[test]
    fn load_from_file() {
        let config = Config::from_file("examples/ceres.conf").unwrap();

        assert_that(&config.default_profile).is_equal_to("prod".to_owned());

        assert_that(&config.profiles).contains_key(String::from("prod"));
        let default_profile = config.profiles.get("prod").unwrap();

        let &Provider::Aws(ref aws) = &default_profile.provider;
        assert_that(&aws.access_key_id).is_equal_to("a key id".to_owned());
        assert_that(&aws.secret_access_key).is_equal_to("an access key".to_owned());
        assert_that(&aws.region).is_equal_to(Region::EuCentral1);
        assert_that(&aws.role_arn).is_equal_to("a_role_arn".to_owned());
    }
}
