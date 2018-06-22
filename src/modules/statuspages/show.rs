use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{Future, Stream};
use futures::future::{join_all, result};
use reqwest::header::Connection;
use reqwest::unstable::async::{Client as ReqwestClient};
use serde_json;
use tokio_core;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::statuspages::{PageStatus, PageStatusResult};
use modules::statuspages::errors::*;
use output::OutputType;
use output::statuspages::*;

pub const NAME: &str = "show";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Query Status Page status for all status pages")
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

fn do_call(args: &ArgMatches, _: &RunConfig, config: &Config) -> Result<()> {
    let status_pages = &config.status_pages;
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::FailedToParseOutputType)?;

    info!("Quering status");
    let mut core = tokio_core::reactor::Core::new()
      .chain_err(|| ErrorKind::FailedToQueryStatusPage)?;
    let client = ReqwestClient::new(&core.handle());

    let queries = status_pages.iter().map(|(name, status_page)| {
        query_page_status(&client, name.to_string(), &status_page.id)
    });
    let work = join_all(queries);
    let result = core.run(work)?;

    info!("Outputting Page Status");
    output_page_status(output_type, &result)?;

    Ok(())
}

fn query_page_status(client: &ReqwestClient, name: String, id: &str) -> impl Future<Item = PageStatusResult, Error = Error> {
    let base_url = format!("https://{}.statuspage.io/api/v2/status.json", id);
    client
        .get(&base_url)
        .header(Connection::close())
        .send()
        .and_then(|res| {
            trace!("Received response with status = {}.", res.status());
            let body = res.into_body();
            body.concat2()
        })
        .map_err(|_| Error::from_kind(ErrorKind::FailedToQueryStatusPage))
        .and_then(|body| {
            trace!("Parsing body.");
            let res = serde_json::from_slice::<PageStatus>(&body)
                .map(|x| PageStatusResult { name, page_status: x })
                .chain_err(|| Error::from_kind(ErrorKind::FailedToQueryStatusPage));
            result(res)
        })
}

fn output_page_status(
    output_type: OutputType,
    status: &[PageStatusResult]
) -> Result<()> {
   let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let output = TableOutputPageStatusResult;

            output
                .output(&mut stdout, status)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Json => {
            let output = JsonOutputPageStatusResult;

            output
                .output(&mut stdout, status)
                .chain_err(|| ErrorKind::FailedOutput)
        },
        OutputType::Plain => {
            let output = PlainOutputPageStatusResult;

            output
                .output(&mut stdout, status)
                .chain_err(|| ErrorKind::FailedOutput)
        },
    }
}

