use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{Future, Stream, collect};
use futures::future::{self, result};
use hyper::header::Headers;
use reqwest::header::{ContentType, Connection};
use reqwest::unstable::async::{Client as ReqwestClient};
use serde_json;
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::stories::XTrackerToken;
use modules::stories::errors::*;
use output::OutputType;

pub const NAME: &str = "prepare";

pub const TASKS: &[&str] = &[
   "Detailed Planning durchführen",
   "Feature Branch (PT-<Ticketnr>) erzeugen",
   "Fachliche Entwicklung mit Deployment für Staging abschließen",
   "PR erstellen",
   "Fachliches Review anfordern",
   "ggf. Review-Kommentare nachpflegen",
   "Dokumentation erstellen",
   "Approval für Prod Merge anfordern",
   "Patch für Prod anwenden",
   "Approval für Deployment anfordern",
   "PR mit Merge nach master schließen",
   "Story fertigstellen (Finished)",
   "Story nach Prod deployment (Delivered)",
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

   let f = get_tasks(&client, project_id, story_id, &token)
      .and_then(|tasks|
         if tasks.len() == 0 || force {
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

#[derive(Debug, Deserialize)]
pub struct TaskResponse {
   pub id: u64,
   pub story_id: u64,
   pub kind: String,
   pub position: u64,
   pub description: String,
   pub complete: bool,
   pub created_at: String,
   pub updated_at: String,
}

fn get_tasks(client: &ReqwestClient, project_id: u64, story_id: u64, token: &str) -> impl Future<Item = Vec<TaskResponse>, Error = Error> {
   let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}/tasks",
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
            let res = serde_json::from_slice::<Vec<TaskResponse>>(&body)
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(res)
        })
}

fn create_task(
   client: &ReqwestClient,
   project_id: u64,
   story_id: u64,
   token: &str,
   position: usize,
   description: &str,
) -> impl Future<Item = TaskResponse, Error = Error> {
      let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}/tasks",
      project_id=project_id,
      story_id=story_id);

   let data = json!({
      "description": format!("{}. {}", position, description),
      "position": position
   }).to_string();

   trace!("Task: {:?}", data);

   client
      .post(&url)
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
         let task = serde_json::from_slice::<TaskResponse>(&body.as_bytes())
            .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
         result(task)
      })
}

