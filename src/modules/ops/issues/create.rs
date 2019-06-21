use clams::console::ask_for_confirmation;
use clap::{App, Arg, ArgMatches, SubCommand};
use hubcaps::{Credentials, Github};
use hubcaps::issues::{Issue, IssueOptions};
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tokio_core::reactor::Core;
use webbrowser;

use config::CeresConfig as Config;
use run_config::RunConfig;
use modules::*;

pub const NAME: &str = "create";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Create ops issue")
            .arg(
                Arg::with_name("title")
                    .short("t")
                    .long("title")
                    .takes_value(true)
                    .required_unless_one(&["browser"])
                    .help("Sets title for issue"),
            )
            .arg(
                Arg::with_name("browser")
                    .long("browser")
                    .conflicts_with_all(&["filename", "interactive"])
                    .required_unless_one(&["filename", "template"])
                    .help("Opens webbrowser to create new issue"),
            )
            .arg(
                Arg::with_name("interactive")
                    .short("i")
                    .long("interactive")
                    .conflicts_with_all(&["browser", "filename"])
                    .required_unless_one(&["browser", "filename"])
                    .help("Opens $EDITOR to write issue contents"),
            )
            .arg(
                Arg::with_name("template")
                    .long("template")
                    .takes_value(true)
                    .conflicts_with_all(&["filename"])
                    .help("Uses this template to pre-fill editor; defaults to config setting"),
            )
            .arg(
                Arg::with_name("filename")
                    .long("filename")
                    .short("f")
                    .takes_value(true)
                    .conflicts_with_all(&["browser", "interactive"])
                    .required_unless_one(&["browser", "interactive"])
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
                Arg::with_name("no-wait")
                    .long("no-wait")
                    .requires("interactive")
                    .help("Do not wait for editor to finish in interactive mode"),
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

    if args.is_present("browser") {
      let template_name: &str = if let Some(ref name) = args.value_of("template") {
        name
      } else {
        &issue_tracker.default_issue_template_name
      };
      let html_url = browse_create_issue(&issue_tracker.github_org, &issue_tracker.github_repo, template_name);
      info!("Opening browser to create new ops issue");
      webbrowser::open(&html_url)
          .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
      return Ok(());
    }

    let title = args.value_of("title").unwrap(); // Safe unwrap
    let labels = args.values_of_lossy("labels").unwrap_or_default();

    let file_path = if args.is_present("interactive") {
        let editor = ::std::env::var_os("EDITOR").unwrap_or_else(|| "vi".to_string().into());
        let mut local_template = PathBuf::new();
        local_template.push(&issue_tracker.local_issue_template_path);
        local_template.push(&issue_tracker.default_issue_template_name);
        let path = create_tempfile_from_template(args.value_of("template"), &local_template.to_string_lossy())?;
        debug!("Editing file {:?}", path);
        edit_file(&path, &editor, !args.is_present("no-wait"))?;
        path
    } else {
        Path::new(args.value_of("filename").unwrap()).to_path_buf() // Safe unwrap
    };
    trace!("Body file path = {:?}", file_path);

    let issue = create_issue(title.to_owned(), &file_path, labels)?;

    debug!("Sending issue {:?}", issue);
    let res = send_issue(&config.github.token, &issue_tracker.github_org, &issue_tracker.github_repo, &issue)
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

fn edit_file(file: &Path, editor: &OsString, wait_for_completion: bool) -> Result<()> {
    let mut ed = Command::new(editor)
        .arg(file.as_os_str())
        .spawn()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let _ = ed.wait()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    if wait_for_completion {
      let _ = ask_for_confirmation("Press <Return> when finished ...", "");
    }

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

fn send_issue(github_token: &str, org: &str, repo: &str, issue: &IssueOptions) -> Result<Issue> {
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

fn browse_create_issue(org: &str, repo: &str, template_name: &str) -> String {
  format!("https://github.com/{}/{}/issues/new?template={}", org, repo, template_name)
}

