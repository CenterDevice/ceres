use std::path::{Component, Path, PathBuf};

// This mod's errors need an individual namespace because the sub_module macro imports the
// module::errors into this scope which leads to name / type conflicts.
mod errors {
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
            FailedToFindResources {
                description("Failed to find resources")
                display("Failed to find resources")
            }
            FailedParseResourcesFromPath(path: String) {
                description("Failed to parse resources from path")
                display("Failed to parse resources from path '{}'", path)
            }
            FailedOutput {
                description("Failed to output")
                display("Failed to output")
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
}

#[derive(Debug, Serialize)]
pub struct Resource {
    pub project: String,
    pub name: String,
}

impl Resource {
    fn to_path<T: AsRef<Path>>(&self, base_dir: T, resources_prefix: T) -> PathBuf {
        let mut p: PathBuf = base_dir.as_ref().to_path_buf();
        p.push(&self.project);
        p.push(resources_prefix);
        p.push(&self.name);
        p
    }

    /// This function assumes that it gets a relative path starting with the project directory
    ///
    /// Example: "logimon/packer/resources/elk_elasticsearch/" instead of "/Users/lukas/Documents/src/ceres/tests/base_dir/logimon/packer/resources/elk_elasticsearch"
    pub fn from_path<P: AsRef<Path>>(path: P) -> errors::Result<Self> {
        let path: &Path = path.as_ref();

        let components: Vec<_> = path.components().collect();
        match components.as_slice() {
            [Component::Normal(project), _, _, Component::Normal(resource)] =>
                Ok( Resource {
                    project: project.to_string_lossy().to_string(),
                    name: resource.to_string_lossy().to_string(),
                } ),
            _ => Err(errors::Error::from_kind(errors::ErrorKind::FailedParseResourcesFromPath(path.to_string_lossy().to_string()))),
        }
    }
}


macro_rules! list_resources {
    ($description:tt,$resources_prefix:tt) => {
        use clap::{App, Arg, ArgMatches, SubCommand};
        use ignore::WalkBuilder;
        use std::path::{Path, PathBuf};

        use config::CeresConfig as Config;
        use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
        use modules::infrastructure::Resource;
        use modules::infrastructure::errors::*;
        use output::OutputType;
        use output::infrastructure::{JsonOutputResourceListResult, OutputResourceListResult, PlainOutputResourceListResult, TableOutputResourceListResult};
        use run_config::RunConfig;

        pub const NAME: &str = "list";

        pub struct SubModule;

        impl Module for SubModule {
            fn build_sub_cli() -> App<'static, 'static> {
                SubCommand::with_name(NAME)
                    .about($description)
                    .arg(
                        Arg::with_name("base-dir")
                            .long("base-dir")
                            .takes_value(true)
                            .help("Overwrites base dir from ceres configuration file"),
                    )
                    .arg(
                        Arg::with_name("output")
                            .long("output")
                            .short("o")
                            .takes_value(true)
                            .default_value("human")
                            .possible_values(&["human", "json", "plain"])
                            .help("Selects output format"),
                    )
            }

            fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> ModuleResult<()> {
                let args = cli_args.unwrap(); // Safe unwrap
                do_call(args, run_config, config)
                    .map_err(|e| ModuleError::with_chain(e, ModuleErrorKind::ModuleFailed(NAME.to_owned())))
            }
        }

        #[allow(unstable_name_collision)] // flatten from itertools
        fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
            let profile = match run_config.active_profile.as_ref() {
                "default" => config.get_default_profile(),
                s => config.get_profile(s),
            }.chain_err(|| ErrorKind::FailedToLoadProfile)?;

            let local_base_dir = if let Some(base_dir) = args.value_of("base-dir") {
                base_dir
            } else {
                profile.local_base_dir.as_ref()
                .ok_or(Error::from_kind(ErrorKind::NoLocalBaseDir))?
            };

            let asps: Result<Vec<_>> = find_resources(local_base_dir)?
                .iter()
                .flat_map(|x| x.strip_prefix(local_base_dir))
                .map(|x| Resource::from_path(x))
                .collect();
            let asps = asps?;

            info!("Outputting resource list");
            output_list(args, run_config, config, &asps)?;

            Ok(())
        }

        fn find_resources<P: AsRef<Path>>(base_dir: P) -> Result<Vec<PathBuf>> {
            let walker = WalkBuilder::new(base_dir).build();

            let resources = walker
                .filter(|x| // Does the path point to a Makefile?
                    x.is_ok() &&
                    x.as_ref().unwrap().path().ends_with("Makefile")
                )
                .filter(|x| // Does the path to the Makefile contain the _resources prefix_
                    x.as_ref().unwrap().path().parent().is_some() && // Safe see above
                    x.as_ref().unwrap().path().parent().unwrap().to_string_lossy().contains($resources_prefix)
                )
                .map(|x|
                    x
                    .map(|d| d.path().parent().unwrap().to_path_buf()) // Safe
                    .map_err(|e| Error::with_chain(e, ErrorKind::FailedToFindResources)))
                .filter(|x| { // Does the parent directory contain a file "project.cfg"
                    if let Ok(x) = x {
                        let mut p = x.clone();
                        p.pop(); // Will be true since at least two parents are guaranteed; see above.
                        p.push("project.cfg");
                        p.exists() && p.is_file()
                    } else {
                        false
                    }
                })
                .collect();

            resources
        }

        fn output_list(
            args: &ArgMatches,
            _: &RunConfig,
            _: &Config,
            resources: &[Resource],
        ) -> Result<()> {
            let output_type = args.value_of("output").unwrap() // Safe
                .parse::<OutputType>()
                .chain_err(|| ErrorKind::FailedToParseOutputType)?;
            let mut stdout = ::std::io::stdout();

            match output_type {
                OutputType::Human => {
                    let output = TableOutputResourceListResult;

                    output
                        .output(&mut stdout, resources)
                        .chain_err(|| ErrorKind::FailedOutput)
                },
                OutputType::Json => {
                    let output = JsonOutputResourceListResult;

                    output
                        .output(&mut stdout, resources)
                        .chain_err(|| ErrorKind::FailedOutput)
                },
                OutputType::Plain => {
                    let output = PlainOutputResourceListResult;

                    output
                        .output(&mut stdout, resources)
                        .chain_err(|| ErrorKind::FailedOutput)
                },
            }
        }

    }
}

macro_rules! build_resource {
    ($description:tt,$resources_prefix:tt,$($command:tt),+) => {
        use clap::{App, Arg, ArgMatches, SubCommand};
        use itertools::Itertools;
        use std::path::Path;
        use std::time::Duration;

        use config::{CeresConfig as Config};
        use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
        use modules::infrastructure::Resource;
        use modules::infrastructure::errors::*;
        use output::OutputType;
        use run_config::RunConfig;
        use tempfile;
        use utils::command::{Command, CommandResult};
        use utils::run;

        pub const NAME: &str = "build";
        const COMMANDS: &'static [&'static str] = &[
            $($command,)*
        ];

        pub struct SubModule;

        impl Module for SubModule {
            fn build_sub_cli() -> App<'static, 'static> {
                SubCommand::with_name(NAME)
                    .about($description)
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
            let resource = Resource {
                project: args.value_of("project").unwrap().to_string(), // Safe
                name: args.value_of("resource").unwrap().to_string(), // Safe
            };
            debug!("Resource path is = '{:#?}'", resource.to_path(local_base_dir, $resources_prefix));

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
                    let cwd = &resource.to_path(local_base_dir, $resources_prefix);
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
                    results.push(res);
                    break;
                } else {
                    results.push(res);
                }
            }
            let results: Vec<_> = results.into_iter().flatten().collect();

            Ok(results)
        }
    }
}

sub_module!("infrastructure", "Do stuff with infrastructure repos", asp, images, resources);

