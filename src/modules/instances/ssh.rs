use clap::{App, Arg, ArgMatches, SubCommand};
use std::net::IpAddr;

use config::{CeresConfig as Config, Provider};
use run_config::RunConfig;
use modules::*;
use provider::{DescribeInstance, InstanceDescriptor};
use utils;

pub const NAME: &str = "ssh";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("SSH to an instance")
            .arg(
                Arg::with_name("instance_id")
                    .required(true)
                    .help("Connects to the instance with this instance id"),
            )
            .arg(
                Arg::with_name("command_args")
                    .multiple(true)
                    .last(true)
                    .help("Executes a command with args on the intance"),
            )
            .arg(
                Arg::with_name("public-ip")
                    .short("p")
                    .long("public-ip")
                    .help("Uses public IP address of instance for connection"),
            )
            .arg(
                Arg::with_name("ssh-opts")
                    .long("ssh-opt")
                    .multiple(true)
                    .takes_value(true)
                    .help("Passes an option to ssh"),
            )
            .arg(
                Arg::with_name("login-name")
                    .long("login-name")
                    .short("l")
                    .takes_value(true)
                    .help("Sets remote login name"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    info!("Querying description for instance.");
    let instance = describe_instance(args, run_config, config)?;

    info!("Executing ssh.");
    ssh_to_instance(args, run_config, config, instance)
}

fn describe_instance(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<InstanceDescriptor> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let Provider::Aws(ref provider) = profile.provider;

    let instance_id = args.value_of("instance_id").unwrap(); // safe

    provider
        .describe_instance(instance_id)
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn ssh_to_instance(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
    instance: InstanceDescriptor,
) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    let ip = if args.is_present("public-ip") {
        instance.public_ip_address
    } else {
        instance.private_ip_address
    };

    let command = args.values_of("command_args")
        .map(|x| x.collect::<Vec<_>>().join(" "));
    let login_name_str: String; // Borrow checker
    let mut ssh_opts: Vec<_> = args.values_of("ssh-opts")
        .map(|x| x.collect::<Vec<_>>())
        .unwrap_or_else(Vec::new);

    if let Some(ref login_name) = profile.ssh_user {
        login_name_str = format!("-l {}", login_name);
        ssh_opts.insert(0, &login_name_str);
    };
    let ssh_opts_str = ssh_opts.join(" ");

    if let Some(ip) = ip {
        let ip_addr: IpAddr = ip.parse()
            .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
        utils::ssh_to_ip_address(
            ip_addr,
            command.as_ref().map(|x| x.as_str()),
            Some(&ssh_opts_str),
        ).chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
    } else {
        Err(Error::from_kind(ErrorKind::ModuleFailed(String::from(
            NAME,
        ))))
    }
}
