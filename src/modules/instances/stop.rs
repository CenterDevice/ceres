use clams::console::ask_for_confirmation;
use clap::{App, Arg, ArgMatches, SubCommand};

use config::{CeresConfig as Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputType;
use output::instances::{JsonOutputStateChanges, OutputStateChanges, TableOutputStatusChanges};
use provider::{StateChange, StopInstances};

pub const NAME: &str = "stop";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("stop instances")
            .arg(
                Arg::with_name("instance_ids")
                    .multiple(true)
                    .required(true)
                    .help("Instance Ids to stop"),
            )
            .arg(
                Arg::with_name("dry")
                    .long("dry")
                    .short("d")
                    .conflicts_with("yes")
                    .help("Makes a dry run without actually terminating the instances"),
            )
            .arg(
                Arg::with_name("force")
                    .long("force")
                    .help("Forces the instances to stop. The instances do not have an opportunity to flush file system caches or file system metadata."),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json"])
                    .help("Selects output format"),
            )
            .arg(
                Arg::with_name("yes")
                    .long("yes-i-really-really-mean-it")
                    .conflicts_with("dry")
                    .help("Don't ask me for veryification"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    info!("Stopping instances.");
    let changes = stop_instances(args, run_config, config)?;

    info!("Outputting instance state changes.");
    output_changes(args, run_config, config, &changes)?;

    Ok(())
}

fn stop_instances(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<Vec<StateChange>> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let Provider::Aws(ref provider) = profile.provider;

    let dry = args.is_present("dry");
    let force = args.is_present("force");
    let yes = args.is_present("yes");

    if force {
        warn!("Going to force stopping instances -- file system caches will not be flushed.");
    }

    match (dry, yes) {
        (true, _) => {
            warn!("Running in dry mode -- no changes will be executed.");
        }
        (false, false) => {
            if !ask_for_confirmation("Going to stop instances. Please type 'yes' to continue: ", "yes").unwrap()
            {
                return Err(Error::from_kind(ErrorKind::ModuleFailed(String::from(
                    NAME,
                ))));
            }
        }
        (false, true) => {}
    }

    let instance_ids: Vec<_> = args.values_of("instance_ids")
        .unwrap() // Safe
        .map(String::from).collect();

    provider
        .stop_instances(dry, force, &instance_ids)
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn output_changes(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    state_changes: &[StateChange],
) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let output = TableOutputStatusChanges {};

            output
                .output(&mut stdout, state_changes)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Json => {
            let output = JsonOutputStateChanges;

            output
                .output(&mut stdout, state_changes)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Plain => {
            unimplemented!("'Plain' output is not supported for this module");
        }
    }
}
