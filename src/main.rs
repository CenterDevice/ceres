extern crate ceres;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate error_chain;

use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};
use std::env;
use std::path::Path;

use ceres::modules::{self, Module};
use ceres::config::Config;

const DEFAULT_CONFIG_FILE_NAME: &str = "ceres.conf";

quick_main!(run);

fn run() -> Result<()> {
    let _ = env_logger::try_init();

    let args = build_cli().get_matches();
    let default_config_file_name = format!("{}/.{}", env::home_dir().unwrap().display(), DEFAULT_CONFIG_FILE_NAME);
    let config = load_config(
        args.value_of("config").unwrap_or(&default_config_file_name))?;

    if let Some(subcommand_name) = args.subcommand_name() {
        if subcommand_name == "completions" {
            return generate_completion(args.subcommand_matches(subcommand_name).unwrap());
        }
    }
    modules::call(&args, &config).map_err(|e| e.into())
}

fn build_cli() -> App<'static, 'static> {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let about = env!("CARGO_PKG_DESCRIPTION");

    App::new(name)
        .setting(AppSettings::SubcommandRequired)
        .version(version)
        .about(about)
        .arg(
            Arg::with_name("config")
                .long("config")
                .takes_value(true)
                .help("Sets config file to use [default: ~/.ceres.conf]")
        )
        .arg(
            Arg::with_name("profile")
                .long("profile")
                .takes_value(true)
                .default_value("default")
                .help("Sets profile to use")
        )
        .subcommand(
            SubCommand::with_name("completions").arg(
                Arg::with_name("shell")
                    .long("shell")
                    .takes_value(true)
                    .possible_values(&["bash", "fish", "zsh"])
                    .required(true)
                    .hidden(true)
                    .help("The shell to generate the script for")
            )
        )
        .subcommand(modules::instances::Instances::build_sub_cli())
}

fn load_config<T: AsRef<Path>>(file_path: T) -> Result<Config> {
    let config = Config::from_file(&file_path)
        .chain_err(|| ErrorKind::FailedToLoadConfigFile(format!("{:#?}", file_path.as_ref())))?;

    Ok(config)
}

fn generate_completion(args: &ArgMatches) -> Result<()> {
    let bin_name = env!("CARGO_PKG_NAME");
    let shell = args.value_of("shell")
        .ok_or_else(|| ErrorKind::CliArgsParsingError("shell argument is missing".to_string()))?;
    build_cli().gen_completions_to(
        bin_name,
        shell.parse::<Shell>().map_err(|_| {
            ErrorKind::CliArgsParsingError("completion script generation failed".to_string())
        })?,
        &mut std::io::stdout(),
    );

    Ok(())
}

error_chain! {
    errors {
        CliArgsParsingError(cause: String) {
            description("Failed to parse CLI arguments")
            display("Failed to parse CLI arguments because {}.", cause)
        }

        FailedToLoadConfigFile(file: String) {
            description("Failed to load config file")
            display("Failed to load config file '{}'", file)
        }
    }
    links {
        Module(ceres::modules::Error, ceres::modules::ErrorKind);
    }
}
