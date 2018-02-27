use clap::{App, ArgMatches, SubCommand};
use std::io::Write;

use config::Config;
use modules::*;
use output::OutputInstances;
use output::table_output::TableOutputInstances;
use provider::{DescribeInstances, InstanceDescriptor};
use provider::aws::Aws;


pub const NAME: &str = "list";

pub struct List;

impl Module for List {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("List instances")
    }

    fn call(cli_args: Option<&ArgMatches>, config: &Config) -> Result<()> {
        // TODO: Move these lines into a factory from cli_args and config
        let provider = Aws { provider_arn: String::from("") };
        let output: TableOutputInstances = Default::default();
        let mut stdout = ::std::io::stdout();

        do_call(&provider, &mut stdout, &output)
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
