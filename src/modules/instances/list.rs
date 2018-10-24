use clap::{App, Arg, ArgMatches, SubCommand};

use config::{CeresConfig as Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputType;
use output::instances::{JsonOutputInstances, OutputInstances, PlainOutputInstances, TableOutputInstances};
use provider::{DescribeInstances, InstanceDescriptor, InstanceDescriptorFields};
use provider::filter;

pub const NAME: &str = "list";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("List instances")
            .arg(
                Arg::with_name("filter")
                    .long("filter")
                    .short("f")
                    .takes_value(true)
                    .help("Filters instances by description fields"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json", "plain"])
                    .help("Selects output format"),
            )
            .arg(
                Arg::with_name("output-options")
                    .long("output-options")
                    .takes_value(true)
                    .default_value(
                        "InstanceId,InstanceType,State,PrivateIpAddress,PublicIpAddress,LaunchTime",
                    )
                    .help("Selects the instance description fields to human output"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    info!("Querying description for instances.");
    let instances = list_instances(args, run_config, config)?;

    info!("Filtering instance descriptions");
    let instances = filter_instances(args, run_config, config, instances)?;

    info!("Outputting instance descriptions");
    output_instances(args, run_config, config, &instances)?;

    Ok(())
}

fn list_instances(
    _: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<Vec<InstanceDescriptor>> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let Provider::Aws(provider) = profile.provider
        .as_ref()
        .ok_or(Error::from_kind(ErrorKind::ConfigMissingInProfile("provider".to_string())))?;

    provider
        .describe_instances()
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn filter_instances(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    instances: Vec<InstanceDescriptor>,
) -> Result<Vec<InstanceDescriptor>> {
    let instances = if let Some(filter_str) = args.value_of("filter") {
        let filter = filter_str
            .parse::<filter::Filter>()
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
        instances
            .into_iter()
            .filter(|i| filter.filter(i))
            .collect::<Vec<_>>()
    } else {
        instances
    };

    Ok(instances)
}

fn output_instances(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    instances: &[InstanceDescriptor],
) -> Result<()> {
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
            let fields =
                fields.map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(NAME.to_owned())))?;
            let output = TableOutputInstances { fields };

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Json => {
            let output = JsonOutputInstances;

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Plain => {
            let fields: ::std::result::Result<Vec<_>, _> = args.value_of("output-options").unwrap() // Safe unwrap
                .split(',')
                .map(|s| s.parse::<InstanceDescriptorFields>())
                .collect();
            let fields =
                fields.map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(NAME.to_owned())))?;
            let output = PlainOutputInstances { fields };

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        }
    }
}

