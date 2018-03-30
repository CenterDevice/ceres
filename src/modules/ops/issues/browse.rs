use clap::{App, Arg, ArgMatches, SubCommand};
use webbrowser;

use config::Config;
use run_config::RunConfig;
use modules::*;

pub const NAME: &str = "browse";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Open ops issues in web browser")
            .arg(
                Arg::with_name("project")
                    .long("project")
                    .short("p")
                    .help("Opens project view instead of issues list"),
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
    let issue_tracker = &profile.issue_tracker;

    let url = if args.is_present("project") {
        info!("Browsing to ops issues project");
        format!("https://github.com/{}/{}/projects/{}", issue_tracker.github_org, issue_tracker.github_repo, issue_tracker.project_number)
    } else {
        info!("Browsing to ops issues");
        format!("https://github.com/{}/{}/issues", issue_tracker.github_org, issue_tracker.github_repo)
    };

    webbrowser::open(&url)
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    Ok(())
}
