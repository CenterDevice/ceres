use clap::{App, Arg, ArgMatches, SubCommand};
use std::time::Duration;
use std::net::IpAddr;

use config::{CeresConfig as Config, Profile, Provider};
use modules::*;
use modules::instances::read_instance_ids;
use output::OutputType;
use provider::{DescribeInstance, InstanceDescriptor};
use run_config::RunConfig;
use utils::command::{Command, CommandResult};
use utils::run;
use utils::ssh;

pub const NAME: &str = "run";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("run command on instances")
            .arg(
                Arg::with_name("instance_ids")
                    .required(true)
                    .multiple(true)
                    .help("Runs command on instances with these instance id; or '-' to read json with instance ids from stdin"),
            )
            .arg(
                Arg::with_name("command_args")
                    .multiple(true)
                    .last(true)
                    .help("Executes a command with args on the instance"),
            )
            .arg(
                Arg::with_name("login-name")
                    .long("login-name")
                    .short("l")
                    .takes_value(true)
                    .help("Sets remote login name"),
            )
            .arg(
                Arg::with_name("no-progress-bar")
                    .long("no-progress-bar")
                    .help("Do not show progressbar during command execution"),
            )
            .arg(
                Arg::with_name("public-ip")
                    .short("p")
                    .long("public-ip")
                    .help("Uses public IP address of instance for connection"),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .takes_value(true)
                    .default_value("human")
                    .possible_values(&["human", "json"])
                    .help("Selects output format"),
            )
            .arg(
                Arg::with_name("show-all")
                    .long("show-all")
                    .help("Show all command results; by default show only results of failed commands"),
            )
            .arg(
                Arg::with_name("ssh-opts")
                    .long("ssh-opt")
                    .multiple(true)
                    .takes_value(true)
                    .help("Passes an option to ssh"),
            )
            .arg(
                Arg::with_name("timeout")
                    .long("timeout")
                    .takes_value(true)
                    .default_value("300")
                    .help("Timeout in sec for command to finish"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    // Parse my args
    let instance_ids: Vec<_> = read_instance_ids(args)?;
    let public_ip = args.is_present("public-ip");

    let ssh_opts: Vec<&str> = args.values_of("ssh-opts").unwrap_or_else(|| Default::default()).collect();
    let remote_commands_args: Vec<&str> = args.values_of("command_args").unwrap_or_else(|| Default::default()).collect();

    let timeout = Duration::from_secs(
        args.value_of("timeout").unwrap() // safe unwrap
        .parse()
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?
    );

    let progress_bar = !args.is_present("no-progress-bar");

    let show_all = args.is_present("show-all");
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    // Run me
    info!("Querying description for instances.");
    let instances = describe_instances(&instance_ids, &profile)?;

    debug!("Building ssh commands.");
    let commands = build_commands(&instances, public_ip, profile.ssh_user.as_ref(), &ssh_opts, &remote_commands_args, timeout)?;

    info!("Running commands.");
    let results = run(commands, progress_bar)?;

    run::output_results(output_type, show_all, results.as_slice())
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    Ok(())
}

fn describe_instances(instance_ids: &[String], profile: &Profile) -> Result<Vec<InstanceDescriptor>> {
    let Provider::Aws(ref provider) = profile.provider;
    let res: Result<Vec<InstanceDescriptor>> = instance_ids.iter().
        map(|id| provider
            .describe_instance(id)
            .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        ).collect();

    res
}

fn build_commands(instances: &[InstanceDescriptor], use_public_ip: bool, login_name: Option<&String>, ssh_opts: &[&str], remote_commands_args: &[&str], timeout: Duration) -> Result<Vec<Command>>  {
    let commands: Result<Vec<_>> = instances.iter()
        .map(|i| {
            let ip_addr: IpAddr = if use_public_ip {
                i.public_ip_address.as_ref()
            } else {
                i.private_ip_address.as_ref()
            }
                .map(|ip| ip.parse())
                // TODO Fix me!
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
            let instance_id = i.instance_id.as_ref()
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
            let command = ssh::build_ssh_command_to_instance(&instance_id, &ip_addr, login_name, &ssh_opts, &remote_commands_args, timeout)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
            trace!("ssh_args for instance {}: {:#?}", instance_id, command);
            Ok(command)
        }).collect();

    commands
}

fn run(commands: Vec<Command>, use_progress_bar: bool) -> Result<Vec<CommandResult>>  {
    if use_progress_bar {
        debug!("Running commands with progress bar.");
        run::run_with_progress(commands)
    } else {
        debug!("Running commands without progress bar.");
        run::run(commands)
    }.chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

