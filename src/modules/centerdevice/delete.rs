use clap::{App, Arg, ArgMatches, SubCommand};
use centerdevice::CenterDevice;
use centerdevice::client::AuthorizedClient;
use failure::Fail;
use std::convert::TryInto;

use config::{CeresConfig as Config, CenterDevice as CenterDeviceConfig};
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::centerdevice::errors::*;

pub const NAME: &str = "delete";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Deletes documents from CenterDevice")
            .arg(Arg::with_name("document-ids")
                .index(1)
                .required(true)
                .multiple(true)
                .help("ID of document to delete")
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> ModuleResult<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
            .map_err(|e| ModuleError::with_chain(e, ModuleErrorKind::ModuleFailed(NAME.to_owned())))
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::FailedToParseCmd("profile".to_string()))?;
    let centerdevice = profile.centerdevice.as_ref().ok_or_else(
        || Error::from_kind(ErrorKind::NoCenterDeviceInProfile)
    )?;

    let document_ids: Vec<&str> = args.values_of("document-ids").unwrap_or_else(Default::default).collect();

    info!("Deleting documents at {}.", centerdevice.base_domain);
    delete_documents(centerdevice, &document_ids)?;
    info!("Successfully deleted documents.");

    Ok(())
}

fn delete_documents(centerdevice: &CenterDeviceConfig, document_ids: &[&str]) -> Result<()> {
    let client: AuthorizedClient = centerdevice.try_into()?;
    let result = client
        .delete_documents(document_ids)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi));
    debug!("Deletion result {:#?}", result);

    result
}
