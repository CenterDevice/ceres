use clams::prelude::{Config as ClamsConfig};
use clap::{App, Arg, ArgMatches, SubCommand};
use centerdevice::{CenterDevice, Client, ClientCredentials, Token};
use centerdevice::client::AuthorizedClient;
use centerdevice::client::auth::{Code, CodeProvider, IntoUrl};
use centerdevice::errors::{Result as CenterDeviceResult};
use failure::Fail;
use std::io;
use std::io::Write;
use std::convert::TryInto;

use config::{CeresConfig as Config, CenterDevice as CenterDeviceConfig, Profile};
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::centerdevice::errors::*;

pub const NAME: &str = "auth";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Authenticate with CenterDevice")
            .arg(
                Arg::with_name("refresh")
                    .short("r")
                    .long("refresh")
                    .help("Just refresh token without re-authentication"),
            )
            .arg(
                Arg::with_name("show")
                    .short("s")
                    .long("show")
                    .required_unless("save")
                    .help("On successful authentication, print the received token to stdout"),
            )
            .arg(
                Arg::with_name("save")
                    .short("S")
                    .long("save")
                    .required_unless("show")
                    .help("On successful authentication, save the received token to configuration file"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> ModuleResult<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
                    .map_err(|e| ModuleError::with_chain(e, ModuleErrorKind::ModuleFailed(NAME.to_owned())))
    }
}

struct CliCodeProvider {}

impl CodeProvider for CliCodeProvider {
    fn get_code<T: IntoUrl>(&self, auth_url: T) -> CenterDeviceResult<Code> {
        let auth_url = auth_url.into_url().expect("Failed to parse auth url");

        println!("Please authenticate at the following URL, wait for the redirect, enter the code into the terminal, and then press return ...");
        println!("\n\t{}\n", auth_url);
        print!("Authentication code: ");
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let code = input.trim();

        let code = Code::new(code.to_string());

        Ok(code)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::FailedToParseCmd("profile".to_string()))?;
    let centerdevice = profile.centerdevice.as_ref().ok_or(
        Error::from_kind(ErrorKind::NoCenterDeviceInProfile)
    )?;

    let token = if args.is_present("refresh") {
        refresh_token(&centerdevice)?
    } else {
        get_token(&centerdevice)?
    };

    debug!("{:#?}", token);

    if args.is_present("show") {
        println!("{:#?}", token);
    }

    if args.is_present("save") {
        save_token(run_config, config, &token)
            .chain_err(|| ErrorKind::FailedToSaveToken)?;
    }

    Ok(())
}

fn get_token(centerdevice: &CenterDeviceConfig) -> Result<Token> {
    let client_credentials = ClientCredentials::new(
        &centerdevice.client_id,
        &centerdevice.client_secret,
    );
    let code_provider = CliCodeProvider {};

    info!("Authenticating with CenterDevice at {}", centerdevice.base_domain);
    let client = Client::new(&centerdevice.base_domain, client_credentials)
        .authorize_with_code_flow(&centerdevice.redirect_uri, &code_provider)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi))?;
    info!("Successfully authenticated.");

    Ok(client.token().clone())
}

fn refresh_token(centerdevice: &CenterDeviceConfig) -> Result<Token> {
    info!("Refreshing token with CenterDevice at {}", centerdevice.base_domain);
    let client: AuthorizedClient = centerdevice.try_into()?;
    let token = client.refresh_access_token()
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi))?;
    info!("Successfully re-energized.");

    Ok(token)
}

fn save_token(run_config: &RunConfig, config: &Config, token: &Token) -> Result<()> {
    let new_config = update_config(run_config, config, token)?;
    new_config.save(run_config.active_config)
        .chain_err(|| ErrorKind::FailedToSaveConfig)?;

    Ok(())
}

fn update_config(run_config: &RunConfig, config: &Config, token: &Token) -> Result<Config> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::FailedToParseCmd("profile".to_string()))?;
    let centerdevice = profile.centerdevice.as_ref().ok_or(
        Error::from_kind(ErrorKind::NoCenterDeviceInProfile)
    )?;

    let centerdevice = CenterDeviceConfig {
        access_token: Some(token.access_token().to_string()),
        refresh_token: Some(token.refresh_token().to_string()),
       ..(*centerdevice).clone()
    };

    let profile = Profile {
        centerdevice: Some(centerdevice),
        ..(*profile).clone()
    };

    let profile_name = match run_config.active_profile.as_ref() {
        "default" => config.default_profile.clone(),
        s => s.to_string(),
    };
    let mut profiles = config.profiles.clone();
    profiles.insert(profile_name, profile);

    let new_config = Config {
        profiles,
        ..(*config).clone()
    };

    Ok(new_config)
}
