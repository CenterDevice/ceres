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
use modules::stories::XTrackerToken;
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
      .and_then(|story|
         if story.estimate.is_none() {
            future::err(Error::from_kind(ErrorKind::StoryIsNotEstimated))
         } else if story.current_state == StoryState::Unstarted || force {
            future::ok(story)
         } else {
            future::err(Error::from_kind(ErrorKind::StoryAlreadyStarted))
         }
      )
      .and_then(|story|
         start_story(&client, project_id, story.id, &token)
      );
   let result = core.run(work)?;

   debug!("{:#?}", result);

   Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum StoryState {
   #[serde(rename = "accepted")]
   Accepted,
   #[serde(rename = "delivered")]
   Delivered,
   #[serde(rename = "finished")]
   Finished,
   #[serde(rename = "started")]
   Started,
   #[serde(rename = "rejected")]
   Rejected,
   #[serde(rename = "planned")]
   Planned,
   #[serde(rename = "unstarted")]
   Unstarted,
   #[serde(rename = "unscheduled")]
   Unscheduled,
}

#[derive(Debug, Deserialize)]
pub struct StoryResponse {
   pub id: u64,
   pub kind: String,
   pub current_state: StoryState,
   pub estimate: Option<f32>,
}

fn get_story(client: &ReqwestClient, project_id: u64, story_id: u64, token: &str) -> impl Future<Item = StoryResponse, Error = Error> {
   let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}",
      project_id=project_id,
      story_id=story_id);
    client
        .get(&url)
        .header(Connection::close())
        .header(XTrackerToken(token.to_string()))
        .send()
        .and_then(|res| {
            trace!("Received response with status = {}.", res.status());
            let body = res.into_body();
            body.concat2()
        })
        .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryPivotalApi))
        .and_then(|body| {
            trace!("Parsing body.");
            let res = serde_json::from_slice::<StoryResponse>(&body)
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(res)
        })
}

fn start_story(
   client: &ReqwestClient,
   project_id: u64,
   story_id: u64,
   token: &str,
) -> impl Future<Item = StoryResponse, Error = Error> {
      let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}",
      project_id=project_id,
      story_id=story_id);

   #[derive(Debug, Serialize)]
   struct StoryRequest {
      current_state: StoryState,
   }
   let data = serde_json::to_string( &StoryRequest { current_state: StoryState::Started } ).unwrap(); // This is safe

   trace!("Story StoryRequest: {:?}", data);

   client
      .put(&url)
      .header(Connection::close())
      .header(ContentType::json())
      .header(XTrackerToken(token.to_string()))
      .body(data)
      .send()
      .and_then(|res| {
          trace!("Received response with status = {}.", res.status());
          let body = res.into_body();
          body.concat2()
      })
      .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryPivotalApi))
      .and_then(|body| {
         let body = String::from_utf8_lossy(&body).to_string();
         trace!("Parsing body {:?}", &body);
         let task = serde_json::from_slice::<StoryResponse>(&body.as_bytes())
            .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
         result(task)
      })
}

