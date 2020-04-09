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
            FailedToParseOutputType {
                description("Failed to parse output type")
                display("Failed to parse output type")
            }
            FailedOutput {
                description("Failed to output")
                display("Failed to output")
            }
            FailedToReadRootCaCert(file: String) {
                description("Failed to read Root CA certificate file")
                display("Failed to read Root CA certificate file '{}'", file)
            }
        }
    }
}

use config::{CenterDevice as CenterDeviceConfig};
use centerdevice::{CenterDevice, Certificate, ClientBuilder, ClientCredentials, Token};
use centerdevice::client::AuthorizedClient;
use centerdevice::client::users::UsersQuery;
use std::convert::TryFrom;
use std::collections::HashMap;
use std::path::Path;

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

        create_centerdevice_client(centerdevice, client_credentials, token)
    }
}

fn create_centerdevice_client<'a>(centerdevice: &'a CenterDeviceConfig, client_credentials: ClientCredentials<'a>, token: Token) -> errors::Result<AuthorizedClient<'a>> {
    let mut client = ClientBuilder::new(&centerdevice.base_domain, client_credentials);
    if let Some(ref root_ca_file) = centerdevice.root_ca {
        let certificate = load_cert_from_file(root_ca_file)?;
        client = client.add_root_certificate(certificate);
    }

    let client = client.build_with_token(token);

    Ok(client)
}

fn load_cert_from_file<P: AsRef<Path>>(path: P) -> errors::Result<Certificate> {
    use self::errors::*;

    let pem = std::fs::read(path.as_ref())
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToReadRootCaCert(path.as_ref().to_string_lossy().to_string())))?;
    let cert = Certificate::from_pem(&pem)
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToReadRootCaCert(path.as_ref().to_string_lossy().to_string())))?;

    debug!("Successfully loaded Root CA from '{}'", path.as_ref().to_string_lossy().to_string());

    Ok(cert)
}

pub(crate) trait AuthorizedClientExt<'a> {
    fn get_user_map(&'a self) -> errors::Result<HashMap<String, String>>;
}

impl<'a> AuthorizedClientExt<'a> for AuthorizedClient<'a> {
    fn get_user_map(self: &'a AuthorizedClient<'a>, ) -> errors::Result<HashMap<String, String>> {
        use self::errors::*;
        use failure::Fail;

        let query = UsersQuery { all: true };
        self
            .search_users(query)
            .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi))
            .map(|x| x.users
                .into_iter()
                .map(|u| (u.id, format!("{} {}", u.first_name, u.last_name)))
                .collect()
            )
    }
}

sub_module!("centerdevice", "Access CenterDevice from the CLI", auth, collections, delete, download, search, upload, users);
