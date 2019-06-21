use clap::{App, Arg, ArgMatches, SubCommand};
use centerdevice::CenterDevice;
use centerdevice::client::AuthorizedClient;
use centerdevice::client::users::{UsersQuery, User};
use failure::Fail;
use std::convert::TryInto;

use config::{CeresConfig as Config, CenterDevice as CenterDeviceConfig};
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::centerdevice::errors::*;
use output::OutputType;
use output::centerdevice::users::*;

pub const NAME: &str = "users";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Search users in CenterDevice")
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .conflicts_with("id")
                .help("Sets username to search"))
            .arg(Arg::with_name("id")
                .long("id")
                .short("i")
                .takes_value(true)
                .conflicts_with("name")
                .help("Sets id to search"))
            .arg(Arg::with_name("include-all")
                .long("all")
                .help("Includes blocked users"))
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
    let centerdevice = profile.centerdevice.as_ref().ok_or_else(
        || Error::from_kind(ErrorKind::NoCenterDeviceInProfile)
    )?;

    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::FailedToParseOutputType)?;

    let query = UsersQuery {
        all: args.is_present("include-all"),
    };
    debug!("{:#?}", query);

    info!("Searching users at {}.", centerdevice.base_domain);
    let mut result = search_users(centerdevice, query)?;
    let found = result.len();

    if let Some(name) = args.value_of("name") {
        let name = name.to_lowercase();
        result = result.into_iter()
            .filter(|x|
                x.first_name.to_lowercase().contains(&name)
                    || x.last_name.to_lowercase().contains(&name)
                    || x.email.to_lowercase().contains(&name)
            )
            .collect()
    }
    if let Some(id) = args.value_of("id") {
        result = result.into_iter()
            .filter(|x| x.id == id)
            .collect()
    }
    info!("Successfully found {} and filtered {} users.", found, result.len());

    info!("Outputting search results");
    output_results(output_type, &result)?;

    Ok(())
}

fn search_users(centerdevice: &CenterDeviceConfig, query: UsersQuery) -> Result<Vec<User>> {
    let client: AuthorizedClient = centerdevice.try_into()?;
    let result = client
        .search_users(query)
        .map(|x| x.users)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi));
    debug!("Search result {:#?}", result);

    result
}

fn output_results(output_type: OutputType, results: &[User]) -> Result<()> {
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let output = TableOutputUsers;

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Json => {
            let output = JsonOutputUsers;

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Plain => {
            let output = PlainOutputUsers;

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::FailedOutput)
        },
    }
}
