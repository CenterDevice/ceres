use clap::{App, Arg, ArgMatches, SubCommand};
use centerdevice::CenterDevice;
use centerdevice::client::AuthorizedClient;
use centerdevice::client::collections::{CollectionsQuery, Collection};
use failure::Fail;
use std::convert::TryInto;
use std::collections::HashMap;

use config::{CeresConfig as Config};
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::centerdevice::AuthorizedClientExt;
use modules::centerdevice::errors::*;
use output::OutputType;
use output::centerdevice::collections::*;

pub const NAME: &str = "collections";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Search collections in CenterDevice")
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .conflicts_with("ids")
                .help("Sets collection name to search"))
            .arg(Arg::with_name("ids")
                .long("ids")
                .short("i")
                .takes_value(true)
                .conflicts_with("name")
                .help("Sets ids to search"))
            .arg(Arg::with_name("include-public")
                .long("all")
                .help("Includes public collections"))
            .arg(Arg::with_name("resolve-ids")
                .long("resolve-ids")
                .short("R")
                .help("Resolves ids"))
            .arg(Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .default_value("human")
                .possible_values(&["human", "json", "plain"])
                .help("Selects output format"))
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
    let centerdevice = profile.centerdevice.as_ref().ok_or(
        Error::from_kind(ErrorKind::NoCenterDeviceInProfile)
    )?;

    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::FailedToParseOutputType)?;

    let mut query = CollectionsQuery::new();
    if args.is_present("include-public") {
        query = query.include_public();
    }
    if let Some(name) = args.value_of("name") {
        query = query.name(name);
    }
    if let Some(ids) = args.values_of("ids") {
        query = query.ids(ids.collect())
    }
    debug!("{:#?}", query);

    let client: AuthorizedClient = centerdevice.try_into()?;

    info!("Searching collections at {}.", centerdevice.base_domain);
    let result = search_collections(&client, query)?;
    info!("Successfully found {} collections.", result.len());

    if args.is_present("resolve-ids") {
        info!("Retrieving users from {}.", centerdevice.base_domain);
        let user_map = client.get_user_map()?;
        info!("Outputting search results with resolved ids");
        output_results(output_type, &result, Some(&user_map))?;
    } else {
        info!("Outputting search results");
        output_results(output_type, &result, None)?;
    }

    Ok(())
}

fn search_collections(client: &AuthorizedClient, query: CollectionsQuery) -> Result<Vec<Collection>> {
    let result = client
        .search_collections(query)
        .map(|x| x.collections)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi));
    debug!("Search result {:#?}", result);

    result
}

fn output_results(output_type: OutputType, results: &[Collection], user_map: Option<&HashMap<String, String>>) -> Result<()> {
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let output = TableOutputCollections { user_map };

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Json => {
            let output = JsonOutputCollections;

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Plain => {
            let output = PlainOutputCollections;

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::FailedOutput)
        },
    }
}
