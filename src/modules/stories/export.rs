use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{Future, Stream};
use futures::future::{self, result};
use reqwest::header::{ContentType, Connection};
use reqwest::unstable::async::{Client as ReqwestClient};
use serde::de::DeserializeOwned;
use serde_json;
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use output::stories::OutputType;
use output::stories::{JsonOutputStory, MarkDownOutputStory, OutputStory};
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::stories::XTrackerToken;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Story {
   pub id: u64,
   pub project_id: Option<u64>,
   pub name: Option<String>,
   pub description: Option<String>,
   pub url: Option<String>,
   pub story_type: Option<StoryType>,
   pub current_state: Option<StoryState>,
   pub estimate: Option<f32>,
   pub created_at: Option<String>,
   pub updated_at: Option<String>,
   pub accepted_at: Option<String>,
   pub requested_by: Person,
   pub owners: Vec<Person>,
   pub labels: Vec<Label>,
   pub tasks: Vec<Task>,
   pub pull_requests: Vec<PullRequest>,
   pub comments: Vec<Comment>,
   pub transitions: Vec<Transition>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum StoryType {
    #[serde(rename = "feature")]
    Feature,
    #[serde(rename = "bug")]
    Bug,
    #[serde(rename = "chore")]
    Chore,
    #[serde(rename = "release")]
    Release,
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub initials: String,
    pub username: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub complete: bool,
    pub position: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub host_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub text: String,
    pub person_id: u64,
    pub commit_identifier: Option<String>,
    pub commit_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub state: StoryState,
    pub occurred_at: String,
    pub performed_by_id: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectMember {
    pub person: Person,
}

fn get_story(client: &ReqwestClient, project_id: u64, story_id: u64, token: &str) -> impl Future<Item = Story, Error = Error> {
   let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/stories/{story_id}?fields=project_id,name,description,requested_by,url,story_type,estimate,current_state,created_at,updated_at,accepted_at,owners,labels,tasks,pull_requests,comments,transitions",
      project_id=project_id,
      story_id=story_id);
   get(&url, client, token)
}

fn get_project_members(client: &ReqwestClient, project_id: u64, token: &str) -> impl Future<Item = Vec<ProjectMember>, Error = Error> {
   let url = format!(
      "https://www.pivotaltracker.com/services/v5/projects/{project_id}/memberships?fields=person",
      project_id=project_id);
   get(&url, client, token)
}

fn get<T>(url: &str, client: &ReqwestClient, token: &str) -> impl Future<Item = T, Error = Error> 
where
    T: DeserializeOwned
{
    client
        .get(url)
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
            let res = serde_json::from_slice::<T>(&body)
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryPivotalApi));
            result(res)
        })
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
