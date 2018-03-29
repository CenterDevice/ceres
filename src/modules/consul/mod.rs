use clap::{App, ArgMatches, SubCommand};
use std::str::FromStr;

use config::Config;
use run_config::RunConfig;
use modules::*;

mod list;

pub const NAME: &str = "consul";

pub struct Consul;

impl Module for Consul {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Do stuff Consul")
            .subcommand(list::List::build_sub_cli())
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let subcommand = cli_args.unwrap();
        let subcommand_name = subcommand
            .subcommand_name()
            .ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
        match subcommand_name {
            list::NAME => list::List::call(subcommand.subcommand_matches(subcommand_name), run_config, config)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
            _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from( subcommand_name)))),
        }
    }
}

#[derive(Debug)]
pub enum NodeField {
    Id,
    Name,
    MetaData(Option<Vec<String>>),
    Address,
    ServicePort,
    ServiceTags,
    ServiceId,
    ServiceName,
}


impl FromStr for NodeField {
    type Err = Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "Id" => Ok(NodeField::Id),
            "Name" => Ok(NodeField::Name),
            s if s.starts_with("MetaData") => {
                let filter = extract_metadata_filter(s);
                Ok(NodeField::MetaData(filter))
            },
            "Address" => Ok(NodeField::Address),
            "ServicePort" => Ok(NodeField::ServicePort),
            "ServiceTags" => Ok(NodeField::ServiceTags),
            "ServiceId" => Ok(NodeField::ServiceTags),
            "ServiceName" => Ok(NodeField::ServiceName),
            _ => Err(Error::from_kind(ErrorKind::ModuleFailed(NAME.to_owned())))
        }
    }
}

fn extract_metadata_filter(metadata_str: &str) -> Option<Vec<String>> {
    if metadata_str.len() < 9 {
        return None;
    };
    let metadata = &metadata_str[9..]; // Safe because we call this function only when the prefix 'Metadata:' has been seen
    let metadata_filter: Vec<_> = metadata.split(':').map(String::from).collect();

    Some(metadata_filter)
}
