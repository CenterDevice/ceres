use clap::{App, ArgMatches, SubCommand};

use config::Config;
use run_config::RunConfig;
use modules::*;

mod list;
mod ssh;
mod terminate;

pub const NAME: &str = "instances";

pub struct Instances;

impl Module for Instances {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Do stuff with instances")
            .subcommand(list::List::build_sub_cli())
            .subcommand(ssh::Ssh::build_sub_cli())
            .subcommand(terminate::Terminate::build_sub_cli())
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let subcommand = cli_args.unwrap();
        let subcommand_name = subcommand.subcommand_name().ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
        match subcommand_name {
            list::NAME => list::List::call(subcommand.subcommand_matches(subcommand_name), run_config, config)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
            ssh::NAME => ssh::Ssh::call(subcommand.subcommand_matches(subcommand_name), run_config, config)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
            terminate::NAME => terminate::Terminate::call(subcommand.subcommand_matches(subcommand_name), run_config, config)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
            _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from(subcommand_name))))
        }
    }
}

