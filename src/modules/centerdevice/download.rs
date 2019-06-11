use clams::prelude::*;
use clap::{App, Arg, ArgMatches, SubCommand};
use centerdevice::{CenterDevice, WithProgress};
use centerdevice::client::AuthorizedClient;
use centerdevice::client::download::Download;
use failure::Fail;
use std::convert::TryInto;
use std::env;
use std::path::{Path, PathBuf};

use config::{CeresConfig as Config, CenterDevice as CenterDeviceConfig};
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::centerdevice::errors::*;

pub const NAME: &str = "download";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Downloads a document from CenterDevice")
            .arg(Arg::with_name("filename")
                .long("filename")
                .short("f")
                .takes_value(true)
                .validator(is_valid_file)
                .help("filename for download; default is original document filename"))
            .arg(Arg::with_name("dirname")
                .long("dirname")
                .short("d")
                .takes_value(true)
                .validator(is_valid_dir)
                .help("directory to download document to; default is current working directory"))
            .arg(Arg::with_name("no-progress")
                .long("no-progress")
                .short("P")
                .help("Do not show progress"))
            .arg(Arg::with_name("document-id")
                .index(1)
                .required(true)
                .help("id of document to download"))

    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> ModuleResult<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
            .map_err(|e| ModuleError::with_chain(e, ModuleErrorKind::ModuleFailed(NAME.to_owned())))
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::FailedToParseCmd("profile".to_string()))?;
    let centerdevice = profile.centerdevice.as_ref().ok_or(
        Error::from_kind(ErrorKind::NoCenterDeviceInProfile)
    )?;

    let document_id = args.value_of("document-id").unwrap(); // Safe
    let dir_path = if let Some(dir) = args.value_of("dirname") {
        PathBuf::from(dir)
    } else {
        env::current_dir().map_err(|_| ErrorKind::FailedToPrepareApiCall)?
    };

    let mut download = Download::new(document_id, &dir_path);
    if let Some(f) = args.value_of("filename") {
        download = download.filename(Path::new(f));
    }

    debug!("{:#?}", &download);

    info!("Downloading from {}.", centerdevice.base_domain);
    let bytes = if args.is_present("no-progress") {
        download_file(centerdevice, download)?
    } else {
        download_file_with_progress(centerdevice, download)?
    };
    info!("Successfully downloaded document with '{}' bytes.", bytes);

    Ok(())
}

fn download_file(centerdevice: &CenterDeviceConfig, download: Download) -> Result<u64> {
    let client: AuthorizedClient = centerdevice.try_into()?;
    let result = client
        .download_file(download)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi));
    debug!("Download result {:#?}", result);

    result
}


fn download_file_with_progress(centerdevice: &CenterDeviceConfig, download: Download) -> Result<u64> {
    let mut progress = Progress::new();
    let client: AuthorizedClient = centerdevice.try_into()?;
    let result = client
        .download_file_with_progress(download, &mut progress)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi));
    debug!("Download result {:#?}", result);

    result
}

fn is_valid_file(filename: String) -> ::std::result::Result<(), String> {
    let path = Path::new(&filename);

    if let Some(parent) = path.parent() {
        if parent.to_string_lossy() != "" {
            let err_msg = format!("The filename '{}' contains a directory '{}'. Use '-d'.", filename, parent.display());
            return Err(err_msg);
        }
    }

    Ok(())
}

fn is_valid_dir(dirname: String) -> ::std::result::Result<(), String> {
    let path = Path::new(&dirname);

    if !path.is_dir() {
        let err_msg = format!("The path '{}' is not a valid directory.", dirname);
        return Err(err_msg);
    }

    Ok(())
}

pub struct Progress {
    progress_bar: ProgressBar,
}

impl Progress {
    fn new() -> Self {
        let progress_bar = ProgressBar::new(0);
        progress_bar.set_style(ProgressStyle::default_clams_bar());
        Progress {
            progress_bar,
        }
    }
}

impl WithProgress for Progress {
    fn setup(&mut self, size: usize) {
        self.progress_bar.set_length(size as u64);
    }

    fn progress(&mut self, amount: usize) {
        self.progress_bar.inc(amount as u64);
    }

    fn finish(&self) {
        self.progress_bar.finish();
    }
}
