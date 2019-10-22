use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{Future, Stream};
use futures::future::{self, result};
use reqwest::header::{ContentType, Connection};
use reqwest::unstable::async::{Client as ReqwestClient};
use serde_json;
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::stories::pivotal_api::*;
use modules::stories::XTrackerToken;
use modules::stories::errors::*;

pub const NAME: &str = "prepare";

pub const TASKS: &[&str] = &[
    "Detail Planning including Risk Assessment",
    "Document Risk Assessment",
    "Prepare Feature Branch",
    "Functional Development including Tests and Documentation",
    "Request Code Review",
    "Request Functional Review",
    "Prepare Deployment",
    "Request Deployment Approval",
    "Merge Pull Request",
    "Execute Deployment",
    "Check Availability in Production",
];

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Prepare a story")
            .arg(
                Arg::with_name("story-id")
                    .multiple(true)
                    .required(true)
                    .help("Id of story to prepare; may start with '#'"),
            )
            .arg(
                Arg::with_name("force")
                    .long("force")
                    .help("Forces creation of tasks even when other tasks already exist"),
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
   let project_id = profile.story_tracker.project_id;

   let story_id = match args.value_of("story-id") {
      Some(x) if x.starts_with('#') => x[1..].parse::<u64>()
         .chain_err(|| ErrorKind::FailedToParseCmd("story-id".to_string())),
      Some(x) => x.parse::<u64>()
         .chain_err(|| ErrorKind::FailedToParseCmd("story-id".to_string())),
      None => Err(Error::from_kind(ErrorKind::FailedToParseCmd("story-id".to_string()))),
   }?;
   let force = args.is_present("force");
   let token = &config.pivotal.token;

   info!("Quering existing tasks");
   let mut core = tokio_core::reactor::Core::new()
     .chain_err(|| ErrorKind::FailedToQueryPivotalApi)?;
   let client = ReqwestClient::new(&core.handle());

   let f = get_story(&client, project_id, story_id, &token)
      .and_then(|story|
         if story.tasks.is_empty() || force {
            future::ok(())
         } else {
            future::err(Error::from_kind(ErrorKind::StoryHasTasksAlready))
         }
      );
   core.run(f)?;

   info!("Creating tasks");
   let result: Result<Vec<_>> = TASKS
      .iter()
      .enumerate()
      .map(|(pos, x)| {
         let f = create_task(&client, project_id, story_id, &token, pos+1, x);
         core.run(f)
      })
      .collect();

   debug!("{:#?}", result);

   Ok(())
}
