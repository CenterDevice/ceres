use clap::{App, Arg, ArgMatches, SubCommand};

use config::{Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputInstances;
use output::instances::{JsonOutputInstances, OutputType, TableOutputInstances};
use provider::{DescribeInstances, InstanceDescriptor, InstanceDescriptorFields};

pub const NAME: &str = "list";

pub struct List;

impl Module for List {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("List instances")
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json"])
                    .help("Selects output format")
            )
            .arg(
                Arg::with_name("output-options")
                    .long("output-options")
                    .takes_value(true)
                    .default_value("InstanceId,InstanceType,State,PrivateIpAddress,PublicIpAddress,LaunchTime")
                    .help("Selects the instance description fields to human output")
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let instances = list_instances(args, run_config, config)?;
    let _ = output_instances(args, run_config, config, &instances)?;

    Ok(())
}

fn list_instances(_: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<Vec<InstanceDescriptor>> {
    let &Provider::Aws(ref provider) = match run_config.active_profile.as_ref() {
        "default" => config.get_default_provider(),
        s => config.get_provider_by_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    provider.describe_instances().chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn output_instances(args: &ArgMatches, _: &RunConfig, _: &Config, instances: &[InstanceDescriptor]) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let fields: ::std::result::Result<Vec<_>, _> = args.value_of("output-options").unwrap() // Safe unwrap
                .split(',')
                .map(|s| s.parse::<InstanceDescriptorFields>())
                .collect();
            let fields = fields
                .map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(NAME.to_owned())))?;
            let output = TableOutputInstances {
                fields,
                tags_filter: Some(vec!["Name".to_owned(), "Intent".to_owned()])
            };

            output.output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        }
        OutputType::Json => {
            let output = JsonOutputInstances;

            output.output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        }
    }
}
