use clap::{App, ArgMatches, SubCommand};

use config::Config;
use run_config::RunConfig;
use modules::*;

mod issues;

pub const NAME: &str = "ops";

pub struct Ops;

impl Module for Ops {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Do ops stuff")
            .subcommand(issues::Issues::build_sub_cli())
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let subcommand = cli_args.unwrap();
        let subcommand_name = subcommand
            .subcommand_name()
            .ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
        match subcommand_name {
            issues::NAME => issues::Issues::call(
                subcommand.subcommand_matches(subcommand_name), run_config, config)
                    .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
            _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from(subcommand_name))))
        }
    }
}

