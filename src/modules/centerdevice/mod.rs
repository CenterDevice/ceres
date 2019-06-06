// This mod's errors need an individual namespace because the sub_module macro imports the
// module::errors into this scope which leads to name / type conflicts.
mod errors {
    error_chain! {
        errors {
            FailedToParseCmd(arg: String) {
                description("Failed to parse command line arguments")
                display("Failed to parse command line argument '{}'", arg)
            }
            NoCenterDeviceInProfile {
                description("No CenterDevice configuration found in selected profile")
            }
            FailedToPrepareApiCall {
                description("Failed to prepare API call")
            }
            FailedToAccessCenterDeviceApi {
                description("Failed to access CenterDevice API")
            }
            FailedToSaveToken {
                description("Failed to save token to configuration file")
            }
            FailedToSaveConfig {
                description("Failed to save configuration file")
            }
            TokenMissing{
                description("No token found in configuration")
            }
        }
    }
}

use config::{CenterDevice as CenterDeviceConfig};
use centerdevice::{Client, ClientCredentials, Token};
use centerdevice::client::AuthorizedClient;
use std::convert::TryFrom;

impl<'a> TryFrom<&'a CenterDeviceConfig> for AuthorizedClient<'a> {
    type Error = errors::Error;

    fn try_from(centerdevice: &'a CenterDeviceConfig) -> std::result::Result<Self, Self::Error> {
        use self::errors::{Error, ErrorKind};

        let client_credentials = ClientCredentials::new(&centerdevice.client_id, &centerdevice.client_secret);
        let access_token = centerdevice.access_token
            .as_ref()
            .ok_or_else(|| Error::from_kind(ErrorKind::TokenMissing))?
            .to_string();
        let refresh_token = centerdevice.refresh_token
            .as_ref()
            .ok_or_else(|| Error::from_kind(ErrorKind::TokenMissing))?
            .to_string();
        let token = Token::new(access_token, refresh_token);
        let client = Client::with_token(&centerdevice.base_domain, client_credentials, token);

        Ok(client)
    }
}

sub_module!("centerdevice", "Access CenterDevice from the CLI", auth, upload);
