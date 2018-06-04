use clap::{App, Arg, ArgMatches, SubCommand};

use config::{CeresConfig as Config, Profile, Provider};
use modules::*;
use output::OutputType;
use run_config::RunConfig;
use utils::run;
use utils::ssh;

pub const NAME: &str = "list";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("list available infrastructure ASPs")
            .arg(
                Arg::with_name("base-dir")
                    .long("base-dir")
                    .takes_value(true)
                    .help("Overwrites base dir from ceres configuration file"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

#[allow(unstable_name_collision)] // flatten from itertools
fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    Ok(())
}

