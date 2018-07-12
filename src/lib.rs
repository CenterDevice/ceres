extern crate clams;
#[macro_use]
extern crate clams_derive;
extern crate chrono;
extern crate chrono_humanize;
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hubcaps;
extern crate ignore;
extern crate itertools;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate log;
extern crate prettytable;
extern crate regex;
extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_ec2;
extern crate rusoto_sts;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate service_world;
extern crate subprocess;
extern crate tempfile;
extern crate tokio_core;
extern crate toml;
extern crate webbrowser;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate spectral;

macro_rules! sub_module {
    ($name:tt,$desciption:tt,$($submodule:tt),+) => {
        use clap::{App, ArgMatches, SubCommand};

        use config::CeresConfig as Config;
        use run_config::RunConfig;
        use modules::*;

        $(
        pub mod $submodule;
        )*

        pub const NAME: &str = $name;

        pub struct SubModule;

        impl Module for SubModule {
            fn build_sub_cli() -> App<'static, 'static> {
                SubCommand::with_name(NAME)
                    .about($desciption)
                    $(
                    .subcommand($submodule::SubModule::build_sub_cli())
                    )*
            }

            fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
                let subcommand = cli_args.unwrap();
                let subcommand_name = subcommand
                    .subcommand_name()
                    .ok_or_else(|| ErrorKind::NoSubcommandSpecified(NAME.to_string()))?;
                match subcommand_name {
                    $(
                    $submodule::NAME => $submodule::SubModule::call(
                        subcommand.subcommand_matches(subcommand_name), run_config, config)
                            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_string())),
                    )*
                    _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from(subcommand_name))))
                }
            }
        }
    }
}

macro_rules! main_module {
    ($($submodule:tt),+) => {

        $(
        pub mod $submodule;
        )*

        pub fn build_sub_cli(app: App<'static, 'static>) -> App<'static, 'static> {
            app
                $(
                .subcommand($submodule::SubModule::build_sub_cli())
                )*
        }

        pub fn call(cli_args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
            let subcommand_name = cli_args
                .subcommand_name()
                .ok_or(ErrorKind::NoCommandSpecified)?;
            let subcommand_args = cli_args.subcommand_matches(subcommand_name);
            match subcommand_name {
                $(
                $submodule::NAME => $submodule::SubModule::call(subcommand_args, run_config, config),
                )*
                _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from(subcommand_name)))),
            }
        }


    }
}

pub mod config;
pub mod modules;
pub mod output;
pub mod provider;
pub mod run_config;
pub mod utils;
