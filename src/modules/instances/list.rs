use clap::{App, Arg, ArgMatches, SubCommand};
use std::io::Write;

use config::{Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputInstances;
use output::table_output::TableOutputInstances;
use provider::{DescribeInstances, InstanceDescriptor, InstanceDescriptorFields};
use provider::aws::Aws;


pub const NAME: &str = "list";

pub struct List;

impl Module for List {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("List instances")
            .arg(
                Arg::with_name("output-options")
                    .long("output-options")
                    .takes_value(true)
                    .default_value("InstanceId,InstanceType,LaunchTime,PrivateIpAddress,PublicIpAddress")
                    .help("Selects the instance description fields to output")
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        // TODO: Move these lines into a factory from cli_args and config
        let Provider::Aws(ref provider) = config.profiles.get(&run_config.active_profile).unwrap().provider;
        let fields: ::std::result::Result<Vec<_>, _> = args.value_of("output-options").unwrap() // Safe unwrap
            .split(',')
            .map(|s| s.parse::<InstanceDescriptorFields>())
            .collect();
        let fields = fields.map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(NAME.to_owned())))?;
        let output = TableOutputInstances { fields };
        let mut stdout = ::std::io::stdout();

        do_call(provider, &mut stdout, &output)
    }
}

fn do_call<T: DescribeInstances, S: Write, U: OutputInstances>(provider: &T, writer: &mut S, output: &U) -> Result<()> {
    let instances = list_instances(provider)?;
    output_instances(writer, output, &instances)?;

    Ok(())
}

fn list_instances<T: DescribeInstances>(provider: &T) -> Result<Vec<InstanceDescriptor>> {
    provider.describe_instances().chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn output_instances<T: Write, S: OutputInstances>(writer: &mut T, output: &S, instances: &[InstanceDescriptor]) -> Result<()> {
    output.output(writer, instances).chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}
