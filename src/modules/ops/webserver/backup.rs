use clap::{App, Arg, ArgMatches, SubCommand};
use itertools::Itertools;
use std::collections::HashMap;
use std::time::Duration;

use config::{CeresConfig as Config, Profile, Provider};
use modules::*;
use output::OutputType;
use provider::{DescribeInstances, InstanceDescriptor};
use provider::filter;
use run_config::RunConfig;
use utils::run;
use utils::ssh;

pub const NAME: &str = "backup";
const DESCRIPTION: &str = "Execute backup script remotely";
const COMMANDS: &'static [&'static str] = &[
    "sudo /usr/local/bin/mysql_backup.sh -s -b -1 -c /root/.mysql-backup-client-options.cnf -k /etc/backup/backup-enc-secret.enc -g 60",
    "sudo /usr/local/bin/wordpress_backup.sh -s -k /etc/backup/backup-enc-secret.enc -g 60"
];

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about(DESCRIPTION)
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
                Arg::with_name("force")
                    .long("force")
                    .help("Force execution even if more than webserver instance have been found."),
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

#[allow(unstable_name_collision)] // flatten from itertools
fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    let force = args.is_present("force");
    let public_ip = args.is_present("public-ip");

    let ssh_opts: Vec<&str> = args.values_of("ssh-opts").unwrap_or_else(|| Default::default()).collect();

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
    info!("Querying instances.");
    let instances = find_instances(&profile)?;
    match instances.len()  {
        0 => {
            error!("Did not find any instances with Intent=webserver");
            return Err(Error::from_kind(ErrorKind::ModuleFailed(String::from(NAME))));
        },
        1 => {},
        e if force => {
            warn!("Found {} instances with Intent=webserver", e);
            warn!("Execution forced !!!");
        },
        e => {
            error!("Found {} instances with Intent=webserver", e);
            return Err(Error::from_kind(ErrorKind::ModuleFailed(String::from(NAME))));
        }
    }

    debug!("Building ssh commands.");
    let commands: Result<Vec<_>> = COMMANDS.iter()
        .map(|c| {
            let command_args: Vec<_> = c.split(' ').collect();
            ssh::build_ssh_command_to_instances(&instances, public_ip, profile.ssh_user.as_ref(), &ssh_opts, &command_args.as_slice(), timeout)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))
        }).collect();
    let commands = commands?;

    info!("Running commands.");
    let results: Result<Vec<_>>  = commands.into_iter()
        .map(|cs|
            run::run(cs, progress_bar)
                .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))
        ).collect();
    let results: Vec<_> = results?.into_iter().flatten().collect();

    run::output_results(output_type, show_all, results.as_slice())
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    Ok(())
}

fn find_instances(profile: &Profile) -> Result<Vec<InstanceDescriptor>> {
    let Provider::Aws(ref provider) = profile.provider;
    let all = provider.describe_instances()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    let mut tags = HashMap::new();
    tags.insert("Intent".to_owned(), Some("webserver"));
    let filter = filter::FilterBuilder::new()
        .tags(tags)
        .build()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let webservers = all
        .into_iter()
        .filter(|i| filter.filter(i))
        .collect();

    Ok(webservers)
}

