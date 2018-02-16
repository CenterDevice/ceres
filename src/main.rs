extern crate ceres;
#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate env_logger;

use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};

quick_main!(run);

fn run() -> Result<()> {
    let _ = env_logger::try_init();

    let args = build_cli().get_matches();

    run_subcommand(&args)
}

fn run_subcommand(args: &ArgMatches) -> Result<()> {
    let subcommand = args.subcommand_name().unwrap();
    let subargs = args.subcommand_matches(subcommand).unwrap();

    match subcommand {
        "completions" => generate_completion(subargs),
        "instances" => {
            let config = args.value_of("profile_arn").unwrap();
            ceres::instances(subargs, config).map_err(|e| e.into())
        }
        _ => Err(ErrorKind::CliArgsParsingError("unknown subcommand".to_string()).into())
    }
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
            Arg::with_name("profile_arn")
                .long("profile_arn")
                .help("Sets profile ARN")
                .takes_value(true)
        )
        .subcommand(
            SubCommand::with_name("completions")
                .arg(
                    Arg::with_name("shell")
                        .long("shell")
                        .help("The shell to generate the script for")
                        .takes_value(true)
                        .possible_values(&["bash", "fish", "zsh"])
                        .required(true)
                )
        )
        .subcommand(
            ceres::subcommand()
        )
}

fn generate_completion(args: &ArgMatches) -> Result<()> {
    println!("{:?}", args);
    let bin_name = env!("CARGO_PKG_NAME");
    let shell = args.value_of("shell").ok_or(ErrorKind::CliArgsParsingError("shell argument is missing".to_string()))?;
    build_cli().gen_completions_to(
        bin_name,
        shell.parse::<Shell>().map_err(|_| ErrorKind::CliArgsParsingError("completion script generation failed".to_string()))?,
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
    }
    links {
        Ceres(ceres::Error, ceres::ErrorKind);
    }
}
