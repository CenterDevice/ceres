extern crate ceres;
extern crate clams;
extern crate clap;
extern crate colored;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use clams::config::{Config, default_locations};
use clams::logging::{Level, ModLevel, init_logging};
use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};
use std::io;

use ceres::config::CeresConfig;
use ceres::modules;
use ceres::run_config::RunConfig;

const DEFAULT_CONFIG_FILE_NAME: &str = "ceres.conf";

fn main() {
    if let Err(ref e) = run() {
        if log_enabled!(log::Level::Error) {
            error!("error: {}", e);
        } else {
            eprintln!("error: {}", e);
        }

        for e in e.iter().skip(1) {
            if log_enabled!(log::Level::Error) {
                error!("caused by: {}", e);
            } else {
                eprintln!("caused by: {}", e);
            }
        }

        if let Some(backtrace) = e.backtrace() {
            if log_enabled!(log::Level::Error) {
                error!("backtrace: {:?}", backtrace);
            } else {
                eprintln!("backtrace: {:?}", backtrace);
            }
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = build_cli().get_matches();
    if args.is_present("no-color") {
        colored::control::set_override(false);
    }

    match args.subcommand_name() {
        Some(subcommand @ "completions") => return generate_completion(args.subcommand_matches(subcommand).unwrap()), // Safe unwrap
        Some("show-example-config") =>return show_example_config(),
        _ => {},
    };

    let mut config_locations = default_locations(DEFAULT_CONFIG_FILE_NAME);
    if let Some(config) = args.value_of("config") {
        config_locations.insert(0, config.into());
    }
    let config = CeresConfig::smart_load(&config_locations)?;

    start_logging(&args, &config)?;

    info!(
        "{} version={}, log level={}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        log::max_level()
    );

    let run_config = RunConfig {
        color: !args.is_present("no-color"),
        active_profile: args.value_of("profile").unwrap().to_owned(), // Safe unwrap
    };
    info!("Active profile={}, default profile={}",
          run_config.active_profile, config.default_profile);

    modules::call(&args, &run_config, &config).map_err(|e| e.into())
}

fn build_cli() -> App<'static, 'static> {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let about = env!("CARGO_PKG_DESCRIPTION");

    let general = App::new(name)
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
            Arg::with_name("no-color")
                .long("no-color")
                .help("Turns off colored output")
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
            SubCommand::with_name("completions")
                .arg(
                    Arg::with_name("shell")
                        .long("shell")
                        .takes_value(true)
                        .possible_values(&["bash", "fish", "zsh"])
                        .required(true)
                        .hidden(true)
                        .help("The shell to generate the script for"),
                )
                .about("Generate shell completion scripts")
        )
        .subcommand(
            SubCommand::with_name("show-example-config")
                .alias("daniel")
                .about("Show an example configuration file")
        );

    modules::build_sub_cli(general)
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

fn start_logging(args: &ArgMatches, config: &CeresConfig) -> Result<()> {
    let verbosity: Level = args.occurrences_of("verbosity").into();

    let default_level: log::LevelFilter = config
        .logging
        .default
        .parse()
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToInitLogging))?;
    let default = Level(default_level);

    let ceres_level: log::LevelFilter = config
        .logging
        .ceres
        .parse()
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToInitLogging))?;
    let ceres = Level(ceres_level);

    let ceres = ::std::cmp::max(ceres, verbosity);

    init_logging(
        io::stderr(),
        !args.is_present("no-color"),
        default,
        vec![ModLevel { module: "ceres".to_owned(), level: ceres }]
    ).chain_err(|| ErrorKind::FailedToInitLogging)?;

    Ok(())
}

fn show_example_config() -> Result<()> {
    let example_config = include_str!("../examples/ceres.conf");

    println!("{}", example_config);

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
        Config(clams::config::ConfigError, clams::config::ConfigErrorKind);
    }
}
