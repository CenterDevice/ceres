use clap::{App, ArgMatches};
use config::Config;
use run_config::RunConfig;

pub mod instances;

pub trait Module {
    fn build_sub_cli() -> App<'static, 'static>;
    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()>;
}

pub fn build_sub_cli(app: App<'static, 'static>) -> App<'static, 'static> {
    app.subcommand(instances::Instances::build_sub_cli())
}

pub fn call(cli_args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let subcommand_name = cli_args
        .subcommand_name()
        .ok_or(ErrorKind::NoCommandSpecified)?;
    let subcommand_args = cli_args.subcommand_matches(subcommand_name);
    match subcommand_name {
        instances::NAME => instances::Instances::call(subcommand_args, run_config, config),
        _ => Err(Error::from_kind(ErrorKind::NoSuchCommand(String::from(
            subcommand_name,
        )))),
    }
}

error_chain! {
    errors {
        NoSuchCommand(command: String) {
            description("no such command")
            display("no such command '{}'", command)
        }

        NoCommandSpecified {
            description("no command specified")
            display("no command specified")
        }

        NoSubcommandSpecified(module_name: String) {
            description("no sub command specified")
            display("no sub command for module {} specified", module_name)
        }

        ModuleFailed(module_name: String) {
            description("module failed")
            display("executing module {} failed", module_name)
        }
    }
}
