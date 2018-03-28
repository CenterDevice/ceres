use clap::{App, Arg, ArgMatches, SubCommand};
use hubcaps::{Credentials, Github};
use hubcaps::issues::{Issue, IssueOptions};
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tokio_core::reactor::Core;
use webbrowser;

use config::Config;
use run_config::RunConfig;
use modules::*;

pub const NAME: &str = "create";

pub struct Create;

impl Module for Create {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Create ops issue")
            .arg(
                Arg::with_name("title")
                    .short("t")
                    .long("title")
                    .takes_value(true)
                    .required(true)
                    .help("Sets title for issue"),
            )
            .arg(
                Arg::with_name("interactive")
                    .short("i")
                    .long("interactive")
                    .conflicts_with("filename")
                    .required_unless("filename")
                    .help("Opens $EDITOR to write issue contents"),
            )
            .arg(
                Arg::with_name("template")
                    .long("template")
                    .takes_value(true)
                    .conflicts_with("filename")
                    .help("Uses this template to pre-fill editor; defaults to config setting"),
            )
            .arg(
                Arg::with_name("filename")
                    .long("filename")
                    .short("f")
                    .takes_value(true)
                    .required_unless("interactive")
                    .help("Sets file name of markdown file to fill issue with"),
            )
            .arg(
                Arg::with_name("labels")
                    .long("label")
                    .short("l")
                    .takes_value(true)
                    .multiple(true)
                    .help("Sets labels for new issue"),
            )
            .arg(
                Arg::with_name("show-in-browser")
                    .long("show-in-browser")
                    .help("Opens newly created issue in web browser"),
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

    let title = args.value_of("title").unwrap(); // Safe unwrap
    let labels = args.values_of_lossy("labels").unwrap_or(Vec::new());

    let file_path = if args.is_present("interactive") {
        let editor = env::var_os("EDITOR").unwrap_or_else(|| "vi".to_string().into());
        let path = create_tempfile_from_template(args.value_of("template"), &issue_tracker.default_issue_template)?;
        debug!("Editing file {:?}", path);
        edit_file(&path, &editor)?;
        path
    } else {
        Path::new(args.value_of("filename").unwrap()).to_path_buf() // Safe unwrap
    };
    trace!("Body file path = {:?}", file_path);

    let issue = create_issue(title.to_owned(), &file_path, labels)?;

    debug!("Sending issue {:?}", issue);
    let res = send_iusse(&config.github.token, &issue_tracker.github_org, &issue_tracker.github_repo, &issue)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    info!("Created issue {}: '{}'", res.number, res.title);
    trace!("Issue = {:?}", res);

    if args.is_present("show-in-browser") {
        webbrowser::open(&res.html_url)
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    }

    Ok(())
}

fn create_tempfile_from_template(template: Option<&str>, default_template: &str) -> Result<PathBuf> {
    let template = if let Some(t) = template {
        t
    } else {
        default_template
    };

    let tmpfile_path = {
        let tmpfile = NamedTempFile::new()
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
        tmpfile.path().to_path_buf()
    };

    trace!("Copying {} to {:?}", template, tmpfile_path);
    ::std::fs::copy(template, &tmpfile_path)
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    Ok(tmpfile_path)
}

fn edit_file(file: &Path, editor: &OsString) -> Result<()> {
    let mut ed = Command::new(editor)
        .arg(file.as_os_str())
        .spawn()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let _ = ed.wait()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    Ok(())
}

fn create_issue(title: String, file_path: &Path, labels: Vec<String>) -> Result<IssueOptions> {
    let mut file = File::open(file_path)
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut body = String::new();
    file.read_to_string(&mut body)
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    let issue = IssueOptions {
        title,
        body: Some(body),
        assignee: None,
        milestone: None,
        labels: labels,
    };

    Ok(issue)
}

fn send_iusse(github_token: &str, org: &str, repo: &str, issue: &IssueOptions) -> Result<Issue> {
  let mut core = Core::new().expect("reactor fail");
  let github = Github::new(
      concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
      Some(Credentials::Token(github_token.to_owned())),
      &core.handle()
  );

  let f = github
    .repo(org, repo)
    .issues()
    .create(issue);
  core.run(f)
    .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))
}

