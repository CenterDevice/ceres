use clap::{App, Arg, ArgMatches, SubCommand};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::time::Duration;

use config::{CeresConfig as Config};
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::infrastructure::asp::Asp;
use output::OutputType;
use run_config::RunConfig;
use tempfile;
use utils::command::{Command, CommandResult};
use utils::run;

pub const NAME: &str = "build";
const COMMANDS: &'static [&'static str] = &[
    "make all"
];

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("build asp for specific project and resource")
            .arg(
                Arg::with_name("base-dir")
                    .long("base-dir")
                    .takes_value(true)
                    .help("Overwrites base dir from ceres configuration file"),
            )
            .arg(
                Arg::with_name("project")
                    .long("project")
                    .short("p")
                    .takes_value(true)
                    .required(true)
                    .help("Sets project"),
            )
            .arg(
                Arg::with_name("resource")
                    .long("resource")
                    .short("r")
                    .takes_value(true)
                    .required(true)
                    .help("Sets resource to build"),
            )
            .arg(
                Arg::with_name("no-progress-bar")
                    .long("no-progress-bar")
                    .help("Do not show progressbar during command execution"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json"])
                    .help("Selects output format"),
            )
            .arg(
                Arg::with_name("show-all")
                    .long("show-all")
                    .help("Show all command results; by default show only results of failed commands"),
            )
            .arg(
                Arg::with_name("timeout")
                    .long("timeout")
                    .takes_value(true)
                    .default_value("300")
                    .help("Timeout in sec for command to finish"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> ModuleResult<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
            .map_err(|e| ModuleError::with_chain(e, ModuleErrorKind::ModuleFailed(NAME.to_owned())))
    }
}

error_chain! {
    errors {
        FailedToLoadProfile {
            description("Failed to load profile")
            display("Failed to load profile")
        }
        NoLocalBaseDir {
            description("No local base directory configured for this profile")
            display("No local base directory configured for this profile")
        }
        FailedToParseDuration {
            description("Failed to parse duration")
            display("Failed to parse duration")
        }
        FailedToParseOutputType {
            description("Failed to parse output type")
            display("Failed to parse output type")
        }
        FailedToBuildCommand {
            description("Failed to build command")
            display("Failed to build command")
        }
        FailedToRunCommand {
            description("Failed to run command")
            display("Failed to run command")
        }
    }
}

impl Asp {
    fn to_path<T: AsRef<Path>>(&self, base_dir: T) -> PathBuf {
        let mut p: PathBuf = base_dir.as_ref().to_path_buf();
        p.push(&self.project);
        p.push("ansible-setup-package/");
        p.push("resources");
        p.push(&self.resource);
        p
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::FailedToLoadProfile)?;

    // Parse my args
    let local_base_dir = if let Some(base_dir) = args.value_of("base-dir") {
        base_dir
    } else {
        profile.local_base_dir.as_ref()
        .ok_or(Error::from_kind(ErrorKind::NoLocalBaseDir))?
    };
    let asp = Asp {
        project: args.value_of("project").unwrap().to_string(), // Safe
        resource: args.value_of("resource").unwrap().to_string(), // Safe
    };
    debug!("Asp path is = '{:#?}'", asp.to_path(local_base_dir));

    let timeout = Duration::from_secs(
        args.value_of("timeout").unwrap() // safe unwrap
        .parse()
        .chain_err(|| ErrorKind::FailedToParseDuration)?
    );

    let progress_bar = !args.is_present("no-progress-bar");

    let show_all = args.is_present("show-all");
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::FailedToParseOutputType)?;

    debug!("Building commands.");
    let commands: Result<Vec<_>> = COMMANDS.iter()
        .map(|c| {
            let cwd = &asp.to_path(local_base_dir);
            build_command(c, cwd, timeout)
        })
        .collect();
    let commands = commands?;

    debug!("Running commands.");
    let results = run_commands(commands, progress_bar)?;

    debug!("Outputting results.");
    run::output_results(output_type, show_all, results.as_slice())
        .chain_err(|| ErrorKind::FailedToRunCommand)?;

    Ok(())
}

fn build_command<T: AsRef<Path>>(command: &str, cwd: T, timeout: Duration) -> Result<Command> {
    let id = command.to_string();

    let mut command_args: Vec<_> = command.split(' ').map(|x| x.to_string()).collect();
    if command_args.len() == 0 {
        return Err(Error::from_kind(ErrorKind::FailedToBuildCommand));
    }
    let cmd = command_args.remove(0);

    let args = if command_args.len() > 0 {
        Some(command_args)
    } else {
        None
    };

    let cwd = cwd.as_ref().to_str().map(|x| x.to_string());
    let log_path = tempfile::NamedTempFile::new()
        .chain_err(|| ErrorKind::FailedToBuildCommand)?
        .path().to_path_buf();

    let c = Command {
        id,
        cmd,
        args,
        cwd,
        log: log_path,
        timeout: Some(timeout),
    };

    Ok(c)
}

#[allow(unstable_name_collision)] // flatten from itertools
fn run_commands(commands: Vec<Command>, progress_bar: bool) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();
    for c in commands.into_iter() {
        let mut res = run::run(vec![c], progress_bar)
            .chain_err(|| ErrorKind::FailedToRunCommand)?;
        if res.iter().filter(|x| !x.exit_status.success()).count() > 0 {
            return Err(Error::from_kind(ErrorKind::FailedToRunCommand));
        }
        results.push(res);
    }
    let results: Vec<_> = results.into_iter().flatten().collect();

    Ok(results)
}
