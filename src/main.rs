extern crate ceres;
#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate env_logger;

use clap::{App, Arg, ArgMatches, Shell};

quick_main!(run);

fn run() -> Result<()> {
    let _ = env_logger::try_init();

    let args = build_cli().get_matches();

    if args.is_present("completions") {
        return generate_completion(&args);
    }

    let profile_arn = args.value_of("profile_arn").unwrap();

    let subcommand = args.subcommand_name().unwrap();
    let subargs = args.subcommand_matches(subcommand).unwrap();
    let subsubcommand = subargs.subcommand_name().unwrap();
    let subsubargs = subargs.subcommand_matches(subsubcommand).unwrap();

    let (tag_key, tag_value) = match subsubargs.value_of("filter") {
        Some(kv) => {
            // cf. https://github.com/rust-lang/rust/issues/23121
            let splits: Vec<_> = kv.split(':').collect();
            match splits.len() {
                1 => (Some(splits[0]), None),
                2 => (Some(splits[0]), Some(splits[1])),
                _ => (None, None),
            }
        }
        None => (None, None),
    };

    let _ = ceres::instances_list(profile_arn, tag_key, tag_value)?;

    Ok(())
}

fn build_cli() -> App<'static, 'static> {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let about = env!("CARGO_PKG_DESCRIPTION");

    App::new(name)
        .version(version)
        .about(about)
        .arg(
            Arg::with_name("profile_arn")
                .long("profile_arn")
                .help("Sets profile ARN")
                .takes_value(true)
                .required_unless("completions"),
        )
        .arg(
            Arg::with_name("completions")
                .long("completions")
                .help("The shell to generate the script for")
                .takes_value(true)
                .possible_values(&["bash", "fish", "zsh"])
                .hidden(true),
        )
        .subcommand(ceres::subcommand())
}

fn generate_completion(args: &ArgMatches) -> Result<()> {
    let bin_name = env!("CARGO_PKG_NAME");
    let shell = args.value_of("completions").unwrap();
    //ok_or(
    //ErrorKind::CliArgsParsingError,
    //)?;
    build_cli().gen_completions_to(
        bin_name,
        shell.parse::<Shell>().unwrap(),
        &mut std::io::stdout(),
    );
    Ok(())
}

error_chain! {
    links {
        Ceres(ceres::Error, ceres::ErrorKind);
    }
}
