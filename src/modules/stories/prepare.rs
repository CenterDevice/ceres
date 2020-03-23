use clap::{App, Arg, ArgMatches, SubCommand};
use futures::Future;
use futures::future;
use reqwest::async::{Client as ReqwestClient};
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use pivotal_api::{create_task, get_story, set_description, Story};
use modules::stories::errors::*;

pub const NAME: &str = "prepare";

pub const TASKS: &[&str] = &[
    "Detail Planning including Risk Assessment",
    "Document Risk Assessment",
    "Create Feature Branch",
    "Development including Tests and Documentation",
    "Create Pull Request and Request Review",
    "Request Functional Acceptance",
    "Prepare Deployment",
    "Request Deployment/Merge Approval",
    "Merge Pull Request",
    "Execute Deployment",
    "Verify Production Delivery",
];

pub const NO_RISK_ASSESSMET: &str = r#"# Risk Assessment

There is no specific risk according to the common risk criteria defined in the DevOps handbook, chapter "Development Process".
"#;

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
   let client = ReqwestClient::new();

   let f = get_story(&client, project_id, story_id, &token)
      .map_err(|e| Error::with_chain(e, ErrorKind::FailedToQueryPivotalApi))
      .and_then(|story|
         if story.tasks.is_empty() || force {
            future::ok(story)
         } else {
            future::err(Error::from_kind(ErrorKind::StoryHasTasksAlready))
         }
      );
   let story: Story = core.run(f)
     .chain_err(|| ErrorKind::FailedToQueryPivotalApi)?;
   debug!("{:#?}", story);

   info!("Creating tasks");
   let result: Result<Vec<_>> = TASKS
      .iter()
      .enumerate()
      .map(|(pos, x)| {
         let f = create_task(&client, project_id, story_id, &token, pos+1, x);
         core.run(f)
            .chain_err(|| ErrorKind::FailedToQueryPivotalApi)
      })
      .collect();
   let result = result?;
   debug!("{:#?}", result);

   info!("Adding description");
   let description = format!("{}\n\n{}\n", story.description.unwrap_or_else(|| "".to_string()), NO_RISK_ASSESSMET);
   let f = set_description(&client, project_id, story_id, &token, &description);
   let result = core.run(f)
     .chain_err(|| ErrorKind::FailedToQueryPivotalApi)?;
   debug!("{:#?}", result);

   Ok(())
}
