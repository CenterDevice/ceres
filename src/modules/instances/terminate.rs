use clap::{App, Arg, ArgMatches, SubCommand};
use colored::Colorize;

use config::{Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputType;
use output::instances::{JsonOutputStateChanges, OutputStateChanges, TableOutputStatusChanges};
use provider::{StateChange, TerminateInstances};
use utils::ask_for_yes_from_stdin;

pub const NAME: &str = "terminate";

pub struct Terminate;

impl Module for Terminate {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("terminate instances")
            .arg(
                Arg::with_name("instance_ids")
                    .multiple(true)
                    .required(true)
                    .help("Instance Ids to terminate"),
            )
            .arg(
                Arg::with_name("dry")
                    .long("dry")
                    .short("d")
                    .conflicts_with("yes")
                    .help("Makes a dry run without actually terminating the instances"),
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
    info!("Terminating instances.");
    let changes = terminate_instances(args, run_config, config)?;

    info!("Outputting instance state changes.");
    output_changes(args, run_config, config, &changes)?;

    Ok(())
}

fn terminate_instances(
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
    let yes = args.is_present("yes");

    match (dry, yes) {
        (true, _) => {
            println!("{}", "Running in dry mode -- no changes will be executed.".yellow());
        }
        (false, false) => {
            if !ask_for_yes_from_stdin(
                "Going to terminate instances. Please type 'yes' to continue: ",
            ).unwrap()
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
        .terminate_instances(dry, &instance_ids)
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
