use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{Future, Stream};
use futures::future::result;
use futures::stream::futures_ordered;
use reqwest::{Certificate, StatusCode};
use reqwest::header::CONNECTION;
use reqwest::async::{Client as ReqwestClient};
use serde_json;
use std::collections::HashMap;
use std::path::Path;
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
   "sales",
   "upload",
];

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthCheck {
   pub name: String,
   pub result: HealthCheckResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum HealthCheckResult {
   Ok(HealthCheckResponse),
   Failed(String),
}

type HealthCheckResponse = HashMap<String, HealthSample>;

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthSample {
   #[serde(rename = "timeStamp")]
   pub time_stamp: Option<i64>,
   #[serde(rename = "samplingTime")]
   pub stampling_time: Option<usize>,
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
      .chain_err(|| ErrorKind::FailedQueryHeatlhCheck("failed to create reactor".to_owned()))?;
   let mut client = ReqwestClient::builder();
   if let Some(ref root_ca_file) = profile.health.root_ca {
      let certificate = load_cert_from_file(root_ca_file)?;
      client = client.add_root_certificate(certificate);
   }
   let client = client.build()
      .map_err(|e| Error::with_chain(e, ErrorKind::FailedQueryHeatlhCheck("failed to create HTTP client".to_owned())))?;

   let queries = ENDPOINTS.iter().map(|name| {
      let url = format!("https://{}.{}/healthcheck", name, base_domain);
      query_health(&client, name, &url)
   });
   let work = futures_ordered(queries).collect();
   let health_checks = core.run(work)?;

   trace!("{:#?}", health_checks);

   info!("Outputting Health Checks");
   output_page_status(output_type, &health_checks)?;

   Ok(())
}

fn load_cert_from_file<P: AsRef<Path>>(path: P) -> Result<Certificate> {
    let pem = std::fs::read(path.as_ref())
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToReadRootCaCert(path.as_ref().to_string_lossy().to_string())))?;
    let cert = Certificate::from_pem(&pem)
        .map_err(|e| Error::with_chain(e, ErrorKind::FailedToReadRootCaCert(path.as_ref().to_string_lossy().to_string())))?;

    debug!("Successfully loaded Root CA from '{}'", path.as_ref().to_string_lossy().to_string());

    Ok(cert)
}

fn query_health(client: &ReqwestClient, name: &'static str, url: &str) -> impl Future<Item = HealthCheck, Error = Error> {
   trace!("Quering health for {}", url);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(CONNECTION, "close".parse().unwrap());

   client
      .get(url)
      .headers(headers)
      .send()
      .map_err(|e| Error::with_chain(e, ErrorKind::FailedQueryHeatlhCheck("failed to request health check from server".to_owned())))
      .and_then(|response| {
         trace!("Received response with status = {}.", response.status());
         let res = if response.status() == StatusCode::OK {
            Ok(response)
         } else {
            let reason = format!("of unexpected status code {} != 200", response.status());
            Err(Error::from_kind(ErrorKind::FailedQueryHeatlhCheck(reason)))
         };
         result(res)
      })
      .and_then(|response| {
         let body = response.into_body();
         body.concat2()
            .map_err(|e| Error::with_chain(e, ErrorKind::FailedQueryHeatlhCheck("failed to read body".to_owned())))
      })
      .and_then(|body| {
         let body = String::from_utf8_lossy(&body).to_string();
         trace!("Parsing body {:?}", &body);
         let res = serde_json::from_slice::<HealthCheckResponse>(&body.as_bytes())
            .map_err(|e| Error::with_chain(e, ErrorKind::FailedQueryHeatlhCheck("failed to parse response".to_owned())));
         result(res)
      })
      .map(move |checks| HealthCheck { name: name.to_string(), result: HealthCheckResult::Ok(checks) } )
      .or_else(move |e| {
         let reason = format!("{}", e);
         Ok(HealthCheck { name: name.to_string(), result: HealthCheckResult::Failed(reason) })
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

