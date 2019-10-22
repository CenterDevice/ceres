use clap::{App, Arg, ArgMatches, SubCommand};
use reqwest::unstable::async::{Client as ReqwestClient};
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use output::stories::OutputType;
use output::stories::{JsonOutputStory, MarkDownOutputStory, OutputStory};
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::stories::pivotal_api::*;
use modules::stories::errors::*;

pub const NAME: &str = "export";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("export a story")
            .arg(
                Arg::with_name("story-id")
                    .required(true)
                    .help("Id of story to prepare; may start with '#'"),
            )
            .arg(
                Arg::with_name("project-id")
                    .short("p")
                    .long("project-id")
                    .takes_value(true)
                    .help("Pivotal Tracker project id; default is profile story tracker id"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("markdown")
                    .possible_values(&["json", "markdown"])
                    .help("Selects output format"),
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
   }.chain_err(|| ErrorKind::FailedToParseCmd("profile".to_string()))?;

   let project_id = match args.value_of("project-id") {
        Some(x) => x.parse::<u64>()
            .chain_err(|| ErrorKind::FailedToParseCmd("project-id".to_string()))?,
        None => profile.story_tracker.project_id
    };

   let story_id = match args.value_of("story-id") {
      Some(x) if x.starts_with('#') => x[1..].parse::<u64>()
         .chain_err(|| ErrorKind::FailedToParseCmd("story-id".to_string())),
      Some(x) => x.parse::<u64>()
         .chain_err(|| ErrorKind::FailedToParseCmd("story-id".to_string())),
      None => Err(Error::from_kind(ErrorKind::FailedToParseCmd("story-id".to_string()))),
   }?;
   let token = &config.pivotal.token;

   info!("Quering story");
   let mut core = tokio_core::reactor::Core::new()
     .chain_err(|| ErrorKind::FailedToQueryPivotalApi)?;
   let client = ReqwestClient::new(&core.handle());

   let work = get_story(&client, project_id, story_id, &token);
   let story = core.run(work)?;
   debug!("{:#?}", story);

   let work = get_project_members(&client, project_id, &token);
   let members = core.run(work)?;
   debug!("{:#?}", members);

   info!("Outputting instance descriptions");
   output_story(args, run_config, config, &story, &members)?;

   Ok(())
}

fn output_story(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    story: &Story,
    members: &[ProjectMember],
) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::FailedToParseCmd("output".to_string()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Json => {
            let output = JsonOutputStory;

            output
                .output(&mut stdout, story, members)
                .chain_err(|| Error::from_kind(ErrorKind::OutputFailed))
        },
        OutputType::MarkDown => {
            let output = MarkDownOutputStory;

            output
                .output(&mut stdout, story, members)
                .chain_err(|| Error::from_kind(ErrorKind::OutputFailed))
        }
    }
}
