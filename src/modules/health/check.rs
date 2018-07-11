use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{Future, Stream};
use futures::future::{join_all, result};
use reqwest::header::Connection;
use reqwest::unstable::async::{Client as ReqwestClient};
use serde_json;
use std::collections::HashMap;
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::health::errors::*;
use output::OutputType;
use output::health::*;

pub const NAME: &str = "check";

pub const ENDPOINTS: &[&str] = &[
   "admin",
   "api",
   "app",
   "auth",
   "public",
   "upload",
];

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheck {
   pub name: String,
   pub checks: HealthCheckResponse,
}

type HealthCheckResponse = HashMap<String, HealthSample>;

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthSample {
   #[serde(rename = "timeStamp")]
   pub time_stamp: i64,
   #[serde(rename = "samplingTime")]
   pub stampling_time: usize,
   #[serde(rename = "value")]
   pub healthy: bool,
}

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Checks health of CenterDevice instance")
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

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
   let profile = match run_config.active_profile.as_ref() {
      "default" => config.get_default_profile(),
      s => config.get_profile(s),
   }.chain_err(|| ErrorKind::FailedToParseCmd("profile".to_string()))?;
   let base_domain = &profile.health.base_domain;

   let output_type = args.value_of("output").unwrap() // Safe
       .parse::<OutputType>()
       .chain_err(|| ErrorKind::FailedToParseOutputType)?;

   info!("Checking Health");
   let mut core = tokio_core::reactor::Core::new()
      .chain_err(|| ErrorKind::FailedQueryHeatlhCheck)?;
   let client = ReqwestClient::new(&core.handle());

   let queries = ENDPOINTS.iter().map(|name| {
      let url = format!("https://{}.{}/healthcheck", name, base_domain);
      query_health(&client, &url)
         .map(move |checks| HealthCheck { name: name.to_string(), checks } )
   });
   let work = join_all(queries);
   let health_checks = core.run(work)?;

   info!("Outputting Health Checks");
   output_page_status(output_type, &health_checks)?;

   Ok(())
}

fn query_health(client: &ReqwestClient, url: &str) -> impl Future<Item = HealthCheckResponse, Error = Error> {
   trace!("Quering health for {}", url);
   client
      .get(url)
      .header(Connection::close())
      .send()
      .and_then(|res| {
         trace!("Received response with status = {}.", res.status());
         let body = res.into_body();
         body.concat2()
      })
      .map_err(|_| Error::from_kind(ErrorKind::FailedQueryHeatlhCheck))
      .and_then(|body| {
         let body = String::from_utf8_lossy(&body).to_string();
         trace!("Parsing body {:?}", &body);
         let res = serde_json::from_slice::<HealthCheckResponse>(&body.as_bytes())
            .chain_err(|| Error::from_kind(ErrorKind::FailedQueryHeatlhCheck));
         result(res)
      })
}

fn output_page_status(
    output_type: OutputType,
    health_checks: &[HealthCheck]
) -> Result<()> {
   let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let output = TableOutputHealthCheck;

            output
                .output(&mut stdout, health_checks)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Json => {
            let output = JsonOutputHealthCheck;

            output
                .output(&mut stdout, health_checks)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Plain => {
            let output = PlainOutputHealthCheck;

            output
                .output(&mut stdout, health_checks)
                .chain_err(|| ErrorKind::FailedOutput)
        },
    }
}

