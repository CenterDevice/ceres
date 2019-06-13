use clap::{App, Arg, ArgMatches, SubCommand};
use centerdevice::CenterDevice;
use centerdevice::client::{AuthorizedClient, ID};
use centerdevice::client::upload::Upload;
use failure::Fail;
use mime::Mime;
use mime_guess;
use std::path::Path;
use std::convert::TryInto;

use config::{CeresConfig as Config, CenterDevice as CenterDeviceConfig};
use run_config::RunConfig;
use modules::{Result as ModuleResult, Error as ModuleError, ErrorKind as ModuleErrorKind, Module};
use modules::centerdevice::errors::*;

pub const NAME: &str = "upload";

pub struct SubModule;

impl Module for SubModule {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("Uploads a document to CenterDevice")
            .arg(Arg::with_name("mime-type")
                .long("mime-type")
                .short("m")
                .takes_value(true)
                .help("Sets the mime type of document; will be guessed if not specified"))
            .arg(Arg::with_name("filename")
                .long("filename")
                .short("f")
                .takes_value(true)
                .help("Sets filename of document different from original filename"))
            .arg(Arg::with_name("title")
                .long("title")
                .takes_value(true)
                .help("Sets title of document"))
            .arg(Arg::with_name("author")
                .long("author")
                .takes_value(true)
                .help("Sets author of document"))
            .arg(Arg::with_name("tags")
                .long("tag")
                .short("t")
                .takes_value(true)
                .multiple(true)
                .help("Sets tag for document"))
            .arg(Arg::with_name("collection")
                .long("collection")
                .short("c")
                .takes_value(true)
                .multiple(true)
                .help("Set collection id to add document to"))
            .arg(Arg::with_name("file")
                .index(1)
                .required(true)
                .help("file to upload")
            )
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

    // This happens here due to the borrow checker.
    let tags: Vec<&str> = args.values_of("tags").unwrap_or_else(|| Default::default()).collect();

    let file_path = args.value_of("file").unwrap(); // Safe

    let mime_type: Mime = if let Some(mt) = args.value_of("mime-type") {
        mt.parse().map_err(|_| ErrorKind::FailedToPrepareApiCall)?
    } else {
        mime_guess::get_mime_type(&file_path)
    };

    let path = Path::new(file_path);
    let mut upload = Upload::new(path, mime_type)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToPrepareApiCall))?;

    if let Some(title) = args.value_of("title") {
        upload = upload.title(title);
    }
    if let Some(author) = args.value_of("author") {
        upload = upload.author(author);
    }
    if !tags.is_empty() {
        upload = upload.tags(&tags);
    }
    debug!("{:#?}", upload);

    info!("Uploading to {}.", centerdevice.base_domain);
    let id = upload_file(centerdevice, upload)?;
    info!("Successfully created document with id '{}'.", id);

    Ok(())
}

fn upload_file(centerdevice: &CenterDeviceConfig, upload: Upload) -> Result<ID> {
    let client: AuthorizedClient = centerdevice.try_into()?;
    let result = client
        .upload_file(upload)
        .map_err(|e| Error::with_chain(e.compat(), ErrorKind::FailedToAccessCenterDeviceApi));
    debug!("Upload result {:#?}", result);

    result
}
