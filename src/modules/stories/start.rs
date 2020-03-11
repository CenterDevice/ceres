use clap::{App, Arg, ArgMatches, SubCommand};
use futures::Future;
use futures::future;
use reqwest::unstable::async::{Client as ReqwestClient};
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use pivotal_api::{get_story, start_story, StoryState};
use modules::stories::errors::*;

pub const NAME: &str = "start";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Start a story")
            .arg(
                Arg::with_name("story-id")
                    .multiple(true)
                    .required(true)
                    .help("Id of story to prepare; may start with '#'"),
            )
            .arg(
                Arg::with_name("force")
                    .long("force")
                    .help("Sets state to started even if current state is not 'unstarted'."),
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

   info!("Quering story state");
   let mut core = tokio_core::reactor::Core::new()
     .chain_err(|| ErrorKind::FailedToQueryPivotalApi)?;
   let client = ReqwestClient::new(&core.handle());

   let work = get_story(&client, project_id, story_id, &token)
      .map_err(|e| Error::with_chain(e, ErrorKind::FailedToQueryPivotalApi))
      .and_then(|story|
         if story.estimate.is_none() {
            future::err(Error::from_kind(ErrorKind::StoryIsNotEstimated))
         } else if story.current_state == Some(StoryState::Unstarted) || force {
            future::ok(story)
         } else {
            future::err(Error::from_kind(ErrorKind::StoryAlreadyStarted))
         }
      )
      .and_then(|story|
         start_story(&client, project_id, story.id, &token)
            .map_err(|e| Error::with_chain(e, ErrorKind::FailedToQueryPivotalApi))
      );
   let result = core.run(work)?;

   debug!("{:#?}", result);

   Ok(())
}

