use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub providers: HashMap<String, Provider>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Provider {
    #[serde(rename = "aws")]
    Aws(AwsProvider)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AwsProvider {
    pub role_arn: String
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;
    use toml;

    #[test]
    fn serialize_deserialize_round_trip() {
        let aws_provider = AwsProvider { role_arn: String::from("a_role_arn") };
        let mut providers = HashMap::new();
        providers.insert("prod@cd".to_owned(), Provider::Aws(aws_provider));
        let config = Config { providers };
        let toml = toml::to_string(&config).unwrap();

        let re_config: Config = toml::from_str(&toml).unwrap();

        assert_that(&re_config).is_equal_to(&config);
    }
}