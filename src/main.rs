extern crate ceres;
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};
use std::env;
use std::path::Path;

use ceres::modules::{self, Module};
use ceres::config::Config;
use ceres::run_config::RunConfig;
use ceres::utils;

const DEFAULT_CONFIG_FILE_NAME: &str = "ceres.conf";

fn main() {
    if let Err(ref e) = run() {
        if log_enabled!(log::Level::Error) { error!("error: {}", e); } else { eprintln!("error: {}", e); }

        for e in e.iter().skip(1) {
            if log_enabled!(log::Level::Error) { error!("caused by: {}", e); } else { eprintln!("caused by: {}", e); }
        }

        if let Some(backtrace) = e.backtrace() {
            if log_enabled!(log::Level::Error) { error!("backtrace: {:?}", backtrace); } else { eprintln!("backtrace: {:?}", backtrace); }
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = build_cli().get_matches();

    if let Some(subcommand_name) = args.subcommand_name() {
        if subcommand_name == "completions" {
            return generate_completion(args.subcommand_matches(subcommand_name).unwrap()); // Safe unwrap
        }
    }

    let default_config_file_name = format!("{}/.{}", env::home_dir().unwrap().display(), DEFAULT_CONFIG_FILE_NAME);
    let config_file_name = args.value_of("config").unwrap_or(&default_config_file_name);
    let config = load_config(&config_file_name)?;

    let _ = init_logging(&args, &config)?;

    fn init_logging(args: &ArgMatches, config: &Config) -> Result<()> {
        let verbosity_level = utils::int_to_log_level(args.occurrences_of("verbosity"));
        let default_level: log::LevelFilter = config.logging.default.parse().map_err(|e| Error::with_chain(e, ErrorKind::FailedToInitLogging))?;
        let ceres_level: log::LevelFilter = config.logging.ceres.parse().map_err(|e| Error::with_chain(e, ErrorKind::FailedToInitLogging))?;
        let ceres_level = ::std::cmp::max(ceres_level, verbosity_level);
        let _ = utils::init_logging(ceres_level, default_level)?;

        Ok(())
    }

    info!("{} version {}, log level={}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), log::max_level());

    let run_config = RunConfig {
        active_profile: args.value_of("profile").unwrap().to_owned(), // Safe unwrap
    };

    modules::call(&args, &run_config, &config).map_err(|e| e.into())
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
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity")
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
        FailedToInitLogging {
            description("Failed to init logging framework")
        }
    }
    links {
        Module(ceres::modules::Error, ceres::modules::ErrorKind);
        Utils(ceres::utils::Error, ceres::utils::ErrorKind);
    }
}
