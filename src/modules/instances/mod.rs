use clap::{App, ArgMatches, SubCommand};

use config::Config;
use modules::*;

mod list;

pub const NAME: &str = "instances";

pub struct Instances;

impl Module for Instances {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Do stuff with instances")
            .subcommand(list::List::build_sub_cli())
    }

    fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
        let subcommand = cli_args.unwrap();
        let subcommand_name = subcommand.subcommand_name().ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
        match subcommand_name {
            list::NAME => list::List::call(subcommand.subcommand_matches(subcommand_name), config)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
            _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from(subcommand_name))))
        }
    }
}

