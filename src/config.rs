use clams::config::prelude::*;
use std::collections::HashMap;

use provider;

#[derive(Config, Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CeresConfig {
    pub default_profile: String,
    pub github:          GitHub,
    pub pivotal:         Pivotal,
    pub logging:         Logging,
    pub status_pages:    HashMap<String, StatusPage>,
    pub profiles:        HashMap<String, Profile>,
}

impl CeresConfig {
    pub fn get_profile(&self, profile_name: &str) -> Result<&Profile> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| ErrorKind::NoSuchProfile(profile_name.to_owned()))?;

        Ok(profile)
    }

    pub fn get_default_profile(&self) -> Result<&Profile> { self.get_profile(&self.default_profile) }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Logging {
    pub default: String,
    pub ceres:   String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GitHub {
    pub token: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Pivotal {
    pub token: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StatusPage {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub ssh_user:       Option<String>,
    pub local_base_dir: Option<String>,
    pub issue_tracker:  IssueTracker,
    pub story_tracker:  StoryTracker,
    pub provider:       Option<Provider>,
    pub consul:         Option<Consul>,
    pub health:         HealthCheck,
    pub centerdevice:   Option<CenterDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IssueTracker {
    pub github_org:                  String,
    pub github_repo:                 String,
    pub project_number:              u64,
    pub default_issue_template_name: String,
    pub local_issue_template_path:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoryTracker {
    pub project_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Provider {
    #[serde(rename = "aws")]
    Aws(provider::aws::Aws),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Consul {
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheck {
    pub base_domain: String,
    pub root_ca:     Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CenterDevice {
    pub client_id:     String,
    pub client_secret: String,
    pub redirect_uri:  String,
    pub base_domain:   String,
    pub access_token:  Option<String>,
    pub refresh_token: Option<String>,
    pub root_ca:       Option<String>,
}

error_chain! {
    errors {
        NoSuchProfile(profile: String) {
            description("No such profile")
            display("No such profile '{}'", profile)
        }
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
            access_key_id:     String::from("a key id"),
            secret_access_key: String::from("an access key"),
            token:             Some("a-token".to_string()),
            region:            Region::EuCentral1,
            role_arn:          String::from("a_role_arn"),
        };
        let issue_tracker = IssueTracker {
            github_org:                  "MyOrg".to_owned(),
            github_repo:                 "MyRepo".to_owned(),
            project_number:              1,
            default_issue_template_name: "some markdown file.md".to_owned(),
            local_issue_template_path:   "some/path".to_owned(),
        };
        let story_tracker = StoryTracker { project_id: 1 };
        let consul = Consul {
            urls: vec!["http://localhost:8500".to_owned(), "http://127.0.0.1:8500".to_owned()],
        };
        let health = HealthCheck {
            base_domain: "instance_domain.com".to_owned(),
            root_ca:     None,
        };
        let centerdevice = CenterDevice {
            client_id:     "aa-bb-cc".to_owned(),
            client_secret: "dd-ee-ff".to_owned(),
            redirect_uri:  "https://exampled.com".to_owned(),
            base_domain:   "centerdevice.de".to_owned(),
            access_token:  None,
            refresh_token: None,
            root_ca:       None,
        };
        let prod_profile = Profile {
            ssh_user: Some("a_user".to_owned()),
            local_base_dir: Some("path/to/your/infrastructure/aws/prod/directory".to_owned()),
            issue_tracker,
            story_tracker,
            provider: Some(Provider::Aws(aws_provider)),
            consul: Some(consul),
            health,
            centerdevice: Some(centerdevice),
        };
        let mut profiles = HashMap::new();
        profiles.insert("prod".to_owned(), prod_profile);
        let status_page = StatusPage {
            id: "123456789".to_owned(),
        };
        let mut status_pages = HashMap::new();
        status_pages.insert("prod".to_owned(), status_page);
        let logging = Logging {
            default: "warn".to_owned(),
            ceres:   "info".to_owned(),
        };
        let github = GitHub {
            token: "a github token".to_owned(),
        };
        let pivotal = Pivotal {
            token: "a pivotal token".to_owned(),
        };
        let config = CeresConfig {
            default_profile: "prod".to_owned(),
            logging,
            github,
            pivotal,
            status_pages,
            profiles,
        };
        let toml = toml::to_string(&config).unwrap();
        eprintln!("toml = {}", toml);

        let re_config: CeresConfig = toml::from_str(&toml).unwrap();

        assert_that(&re_config).is_equal_to(&config);
    }

    #[test]
    fn load_from_file() {
        let config = CeresConfig::from_file("examples/ceres.conf").unwrap();

        assert_that(&config.default_profile).is_equal_to("staging".to_owned());

        assert_that(&config.profiles).contains_key(String::from("staging"));
        let default_profile = config.profiles.get("staging").unwrap();

        let profile = default_profile;
        assert_that(&profile.ssh_user)
            .is_some()
            .is_equal_to("a_user".to_owned());
        assert_that(&profile.local_base_dir)
            .is_some()
            .is_equal_to("path/to/your/infrastructure/aws/prod/directory".to_owned());

        let &Provider::Aws(aws) = &default_profile.provider.as_ref().expect("no AWS provider");
        assert_that(&aws.access_key_id).is_equal_to("XXXXX".to_owned());
        assert_that(&aws.secret_access_key).is_equal_to("XXXXX".to_owned());
        assert_that(&aws.region).is_equal_to(Region::EuCentral1);
        assert_that(&aws.role_arn).is_equal_to("arn:aws:iam::XXXXX:role/OrganizationAccountAccessRole".to_owned());
    }
}
