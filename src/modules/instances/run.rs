use clams::prelude::*;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::net::IpAddr;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::thread;
use tempfile;

use config::{CeresConfig as Config, Provider};
use modules::*;
use modules::instances::read_instance_ids;
use output::OutputType;
use output::instances::{JsonOutputCommandResults, OutputCommandResults, TableOutputCommandResults};
use provider::{DescribeInstance, InstanceDescriptor};
use run_config::RunConfig;
use utils::command::{Command, CommandResult, ExitStatus};

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
    info!("Querying description for instances.");
    let instances = describe_instances(args, run_config, config)?;

    let timeout = Duration::from_secs(
        args.value_of("timeout").unwrap() // safe unwrap
        .parse()
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?
    );

    let commands: Vec<_> = build_commands(args, run_config, config, instances, timeout)?;

    let results = if args.is_present("no-progress-bar") {
        info!("Running commands.");
        run(commands)
    } else {
        info!("Running commands with progress bar.");
        run_with_progress(commands)
    }.chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;

    output_results(args, run_config, config, results.as_slice())?;

    Ok(())
}

fn describe_instances(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<Vec<InstanceDescriptor>> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let Provider::Aws(ref provider) = profile.provider;

    let instance_ids: Vec<_> = read_instance_ids(args)?;

    let res: Result<Vec<InstanceDescriptor>> = instance_ids.iter().
        map(|id| provider
            .describe_instance(id)
            .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        ).collect();

    res
}

fn build_commands(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
    instances: Vec<InstanceDescriptor>,
    timeout: Duration
) -> Result<Vec<Command>> {
    instances.into_iter()
    .map(|i| {
        let ssh_args = build_ssh_arguments(args, run_config, config, &i)
            .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
        let instance_id = i.instance_id
            .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
        trace!("ssh_args for instance {}: {:#?}", instance_id, ssh_args);
        let log_path = tempfile::NamedTempFile::new().unwrap().path().to_path_buf();
        let c = Command {
            id: instance_id,
            cmd: "ssh".to_owned(),
            args: Some(ssh_args),
            log: log_path,
            timeout: Some(timeout),
        };
        Ok(c)
    }).collect()
}

fn build_ssh_arguments(
    args: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
    instance: &InstanceDescriptor,
) -> Result<Vec<String>> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    let ip = if args.is_present("public-ip") {
        instance.public_ip_address.clone()
    } else {
        instance.private_ip_address.clone()
    };

    let mut ssh_args = Vec::new();

    if let Some(ref login_name) = profile.ssh_user {
        ssh_args.push("-l".to_owned());
        ssh_args.push(login_name.to_owned());
    };

    let mut ssh_opts: Vec<_> = args.values_of("ssh-opts")
        .map(|x| x
             .map(|s| s.to_owned())
             .collect::<Vec<_>>()
        ).unwrap_or_else(Vec::new);
    ssh_args.append(&mut ssh_opts);

    if let Some(ip) = ip {
        let ip_addr: IpAddr = ip.parse()
            .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))?;
        ssh_args.push(ip_addr.to_string());
    } else {
        return Err(Error::from_kind(ErrorKind::ModuleFailed(String::from(NAME))))
    }

    let mut command_args = args.values_of("command_args")
        .map(|x| x
             .map(|s| s.to_owned())
             .collect::<Vec<_>>()
        ).unwrap_or_else(Vec::new);
    ssh_args.append(&mut command_args);

    Ok(ssh_args)
}


fn run(commands: Vec<Command>) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();

    for cmd in commands.into_iter() {
        let (sender, receiver) = channel();
        results.push(receiver);

        let _ = thread::spawn(move || {
            let res = cmd.run(None::<fn()>);
            sender.send(res).unwrap();
        });
    }
    let res = results.iter()
        .map(|r|
            r.recv().unwrap()
                .map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(String::from(NAME))))
        )
        .collect();

    res
}

fn run_with_progress(commands: Vec<Command>) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();
    let m = MultiProgress::new();

    for cmd in commands.into_iter() {
        let (sender, receiver) = channel();
        results.push(receiver);

        let spinner_style = ProgressStyle::default_clams_spinner();
        let pb = m.add(ProgressBar::new_spinner());
        pb.set_style(spinner_style);
        pb.set_prefix(&format!("{}", &cmd.id));
        pb.set_message(&cmd.cmd);

        let log_path = cmd.log.clone();
        let _ = thread::spawn(move || {
            let progress = || {
                let line = File::open(log_path.clone()).unwrap().read_last_line().unwrap();

                pb.set_message(&format!("Running: {}", line));
                pb.inc(1);
            };

            let res = cmd.run(Some(progress));

            let finish_msg = match &res {
                &Ok( CommandResult { id: _, log: _, exit_status: ExitStatus::Exited(0) } ) => format!("{}.", "Done".green()),
                &Ok( CommandResult { id: _, log: _, exit_status: ExitStatus::Exited(n) } ) => format!("{} with exit status {}.", "Failed".red(), n),
                &Ok(ref result) => format!("{} with {:?}", "Failed".red(), result.exit_status),
                &Err(ref e) => format!("{} ({:?})", "Error".red(), e),
            };
            pb.finish_with_message(&finish_msg);

            sender.send(res).unwrap();
        });
    }
    m.join().unwrap();

    let res = results.iter()
        .map(|r|
            r.recv().unwrap()
                .map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(String::from(NAME))))
        )
        .collect();

    res
}

fn output_results(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    results: &[CommandResult],
) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let show_all = args.is_present("show-all");
            let output = TableOutputCommandResults { show_all };

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Json => {
            let output = JsonOutputCommandResults;

            output
                .output(&mut stdout, results)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Plain => {
            unimplemented!("'Plain' output is not supported for this module");
        }
    }
}
