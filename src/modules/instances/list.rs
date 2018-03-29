use clap::{App, Arg, ArgMatches, SubCommand};

use config::{Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputType;
use output::instances::{JsonOutputInstances, OutputInstances, TableOutputInstances};
use provider::{DescribeInstances, InstanceDescriptor, InstanceDescriptorFields};

pub const NAME: &str = "list";

pub struct List;

impl Module for List {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("List instances")
            .arg(
                Arg::with_name("filter")
                    .long("filter")
                    .short("f")
                    .takes_value(true)
                    .help("Filters instances by description fields"),
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
                Arg::with_name("output-options")
                    .long("output-options")
                    .takes_value(true)
                    .default_value(
                        "InstanceId,InstanceType,State,PrivateIpAddress,PublicIpAddress,LaunchTime",
                    )
                    .help("Selects the instance description fields to human output"),
            )
    }

    fn call(cli_args: Option<&ArgMatches>, run_config: &RunConfig, config: &Config) -> Result<()> {
        let args = cli_args.unwrap(); // Safe unwrap
        do_call(args, run_config, config)
    }
}

fn do_call(args: &ArgMatches, run_config: &RunConfig, config: &Config) -> Result<()> {
    info!("Querying description for instances.");
    let instances = list_instances(args, run_config, config)?;

    info!("Filtering instance descriptions");
    let instances = filter_instances(args, run_config, config, instances)?;

    info!("Outputting instance descriptions");
    output_instances(args, run_config, config, &instances)?;

    Ok(())
}

fn list_instances(
    _: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<Vec<InstanceDescriptor>> {
    let profile = match run_config.active_profile.as_ref() {
        "default" => config.get_default_profile(),
        s => config.get_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let Provider::Aws(ref provider) = profile.provider;

    provider
        .describe_instances()
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
}

fn filter_instances(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    instances: Vec<InstanceDescriptor>,
) -> Result<Vec<InstanceDescriptor>> {
    let instances = if let Some(filter_str) = args.value_of("filter") {
        let filter = filter_str
            .parse::<filter::Filter>()
            .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
        instances
            .into_iter()
            .filter(|i| filter.filter(i))
            .collect::<Vec<_>>()
    } else {
        instances
    };

    Ok(instances)
}

fn output_instances(
    args: &ArgMatches,
    _: &RunConfig,
    _: &Config,
    instances: &[InstanceDescriptor],
) -> Result<()> {
    let output_type = args.value_of("output").unwrap() // Safe
        .parse::<OutputType>()
        .chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;
    let mut stdout = ::std::io::stdout();

    match output_type {
        OutputType::Human => {
            let fields: ::std::result::Result<Vec<_>, _> = args.value_of("output-options").unwrap() // Safe unwrap
                .split(',')
                .map(|s| s.parse::<InstanceDescriptorFields>())
                .collect();
            let fields =
                fields.map_err(|e| Error::with_chain(e, ErrorKind::ModuleFailed(NAME.to_owned())))?;
            let output = TableOutputInstances { fields };

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Json => {
            let output = JsonOutputInstances;

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        },
        OutputType::Plain => {
            unimplemented!("'Plain' output is not supported for this module");
        }
    }
}

mod filter {
    use regex::Regex;

    use std::collections::HashMap;
    use std::str::FromStr;
    use provider::{InstanceDescriptor, InstanceDescriptorFields};

    macro_rules! filter_builder {
        ($($field:tt),+) => {
            pub struct FilterBuilder<'a> {
                $($field: Option<&'a str>),*,
                block_device_mappings: Option<&'a str>,
                security_groups: Option<&'a str>,
                tags: Option<HashMap<String, Option<&'a str>>>
            }

            impl < 'a > FilterBuilder < 'a > {
                pub fn new() -> Self {
                    FilterBuilder {
                        $($field: None),*,
                        block_device_mappings: None,
                        security_groups: None,
                        tags: None
                    }
                }

                $(
                pub fn $field(mut self, $field: &'a str) -> Self {
                    self.$field = Some($field);
                    self
                }
                )*

                pub fn block_device_mappings(mut self, block_device_mappings: &'a str) -> Self {
                    self.block_device_mappings = Some(block_device_mappings);
                    self
                }

                pub fn security_groups(mut self, security_groups: &'a str) -> Self {
                    self.security_groups = Some(security_groups);
                    self
                }

                pub fn tags(mut self, tags: HashMap<String, Option<&'a str>>) -> Self {
                    self.tags = Some(tags);
                    self
                }

                pub fn build(self) -> Result<Filter> {
                    let filter = Filter {
                        $(
                        $field: if let Some(re) = self.$field {
                            Some(Regex::new(re)
                                .chain_err(|| ErrorKind::FilterRegexError(re.to_owned(), "$field".to_owned()))?)
                        } else {
                            None
                        }
                        ),*,

                        block_device_mappings: if let Some(re) = self.block_device_mappings {
                            Some(Regex::new(re)
                                .chain_err(|| ErrorKind::FilterRegexError(re.to_owned(), "block_device_mappings".to_owned()))?)
                        } else {
                            None
                        },

                        security_groups: if let Some(re) = self.security_groups {
                            Some(Regex::new(re)
                                .chain_err(|| ErrorKind::FilterRegexError(re.to_owned(), "security_groups".to_owned()))?)
                        } else {
                            None
                        },

                        tags: if let Some(tags) = self.tags {
                            let h = tags.into_iter().map(|(k, v)| (
                                k,
                                if let Some(v) = v { Regex::new(v).ok() } else { None }
                            )).collect();
                            Some(h)
                        } else {
                            None
                        },
                    };

                    Ok(filter)
                }
            }

            pub struct Filter {
                $($field: Option<Regex>),*,
                security_groups: Option<Regex>,
                block_device_mappings: Option<Regex>,
                tags: Option<HashMap<String, Option<Regex>>>
            }

           impl Filter {
                pub fn filter(&self, instance: &InstanceDescriptor) -> bool {
                    $(
                    if let Some(ref re) = self.$field {
                        if !re.is_match(instance.$field.as_ref().unwrap()) { return false };
                    }
                    )*

                    if let Some(ref re) = self.block_device_mappings {
                        for s in instance.block_device_mappings.as_ref().unwrap() {
                            if !re.is_match(s) { return false };
                        }
                    }

                    if let Some(ref re) = self.security_groups {
                        for s in instance.security_groups.as_ref().unwrap() {
                            if !re.is_match(s) { return false };
                        }
                    }

                    match (&self.tags, &instance.tags) {
                        (&Some(ref filter_tags), &Some(ref instance_tags)) => {
                            for tag in filter_tags.keys() {
                                if !instance_tags.contains_key(tag) { return false };
                                // We now know that the instance has the the desired tag
                                match (&filter_tags[tag], &instance_tags[tag]) {
                                    (&Some(ref re), &Some(ref value)) if !re.is_match(value)  => return false,
                                    (&Some(_), &None) => return false,
                                    _ => {}
                                }
                            }
                        },
                        (&Some(_), &None) => return false,
                        _ => {},

                    };

                    true
                }
            }
        }
    }

    filter_builder!(
        hypervisor,
        iam_instance_profile,
        image_id,
        instance_id,
        instance_type,
        monitoring,
        placement,
        private_dns_name,
        private_ip_address,
        public_dns_name,
        public_ip_address,
        root_device_name,
        root_device_type,
        state,
        state_reason,
        virtualization_type,
        vpc_id
    );

    impl FromStr for Filter {
        type Err = Error;

        fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
            let tags = s.split(',');
            let kvs: Result<Vec<(&str, Option<&str>)>> = tags.map(|tag| {
                let mut splits: Vec<_> = tag.splitn(2, '=').collect();
                match splits.len() {
                    2 => Ok((splits.remove(0), Some(splits.remove(0)))),
                    1 => Ok((splits.remove(0), None)),
                    _ => Err(Error::from_kind(ErrorKind::FilterParsingFailed(
                        s.to_owned(),
                        "splitting fields failed".to_owned(),
                    ))),
                }
            }).collect();
            let kvs = kvs?;

            let mut f_builder = FilterBuilder::new();
            for (key, value) in kvs {
                if let Some(v) = value {
                    match key.parse::<InstanceDescriptorFields>().chain_err(|| {
                        Error::from_kind(ErrorKind::FilterParsingFailed(
                            s.to_owned(),
                            "parsing instance descriptor field failed".to_owned(),
                        ))
                    })? {
                        InstanceDescriptorFields::BlockDeviceMappings => {
                            f_builder = f_builder.block_device_mappings(v)
                        }
                        InstanceDescriptorFields::Hypervisor => f_builder = f_builder.hypervisor(v),
                        InstanceDescriptorFields::IamInstanceProfile => {
                            f_builder = f_builder.iam_instance_profile(v);
                        }
                        InstanceDescriptorFields::ImageId => {
                            f_builder = f_builder.image_id(v);
                        }
                        InstanceDescriptorFields::InstanceId => {
                            f_builder = f_builder.instance_id(v);
                        }
                        InstanceDescriptorFields::InstanceType => {
                            f_builder = f_builder.instance_type(v);
                        }
                        InstanceDescriptorFields::LaunchTime => {
                            /* A string based time matcher does not make sense */
                        }
                        InstanceDescriptorFields::Monitoring => {
                            f_builder = f_builder.monitoring(v);
                        }
                        InstanceDescriptorFields::Placement => {
                            f_builder = f_builder.placement(v);
                        }
                        InstanceDescriptorFields::PrivateDnsName => {
                            f_builder = f_builder.private_dns_name(v);
                        }
                        InstanceDescriptorFields::PrivateIpAddress => {
                            f_builder = f_builder.private_ip_address(v);
                        }
                        InstanceDescriptorFields::PublicDnsName => {
                            f_builder = f_builder.public_dns_name(v);
                        }
                        InstanceDescriptorFields::PublicIpAddress => {
                            f_builder = f_builder.public_ip_address(v);
                        }
                        InstanceDescriptorFields::RootDeviceName => {
                            f_builder = f_builder.root_device_name(v);
                        }
                        InstanceDescriptorFields::RootDeviceType => {
                            f_builder = f_builder.root_device_type(v);
                        }
                        InstanceDescriptorFields::SecurityGroups => {
                            f_builder = f_builder.security_groups(v);
                        }
                        InstanceDescriptorFields::State => {
                            f_builder = f_builder.state(v);
                        }
                        InstanceDescriptorFields::StateReason => {
                            f_builder = f_builder.state_reason(v);
                        }
                        InstanceDescriptorFields::Tags(_) => {
                            f_builder = f_builder.tags(parse_tags_filter_to_hash(v)?);
                        }
                        InstanceDescriptorFields::VirtualizationType => {
                            f_builder = f_builder.virtualization_type(v);
                        }
                        InstanceDescriptorFields::VpcId => {
                            f_builder = f_builder.vpc_id(v);
                        }
                    }
                }
            }

            f_builder.build().chain_err(|| {
                Error::from_kind(ErrorKind::FilterParsingFailed(
                    s.to_owned(),
                    "building filter failed".to_owned(),
                ))
            })
        }
    }

    fn parse_tags_filter_to_hash(tags_filter: &str) -> Result<HashMap<String, Option<&str>>> {
        let mut hm = HashMap::new();
        for tag in tags_filter.split(':') {
            let mut kv: Vec<_> = tag.split('=').collect();
            match kv.len() {
                2 => hm.insert(kv.remove(0).to_owned(), Some(kv.remove(0))),
                1 => hm.insert(kv.remove(0).to_owned(), None),
                _ => {
                    return Err(Error::from_kind(ErrorKind::FilterParsingFailed(
                        tags_filter.to_owned(),
                        "splitting fields failed".to_owned(),
                    )))
                }
            };
        }

        Ok(hm)
    }

    error_chain! {
        errors {
            FilterRegexError(re: String, field: String) {
                description("Failed to build reg exp.")
                display("Failed to build reg exp '{}' for field '{}'", re, field)
            }
            FilterParsingFailed(s: String, reason: String) {
                description("Failed to parse Filter from String.")
                display("Failed to parse Filter from String '{}' becaue {}", s, reason)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use spectral::prelude::*;
        use std::collections::HashMap;

        #[test]
        fn parse_filter_no_tags_okay() {
            let filter_arg = "InstanceId=i-.*,State=stopped";
            let _ = filter_arg.parse::<Filter>().unwrap();
        }

        #[test]
        fn parse_filter_with_tags_okay() {
            let filter_arg = "InstanceId=i-.*,Tags=Name:AnsibleHostGroup=batch_.*,State=stopped";
            let _ = filter_arg.parse::<Filter>().unwrap();
        }

        #[test]
        fn parse_tags_filter_to_hash_okay() {
            let tags_filter = "Name:AnsibleHostGroup=batch_.*";

            let hm = parse_tags_filter_to_hash(tags_filter);

            assert_that(&hm)
                .is_ok()
                .contains_entry("Name".to_owned(), None);
            assert_that(&hm)
                .is_ok()
                .contains_entry("AnsibleHostGroup".to_owned(), Some("batch_.*"));
        }

        #[test]
        fn filter_instance_with_invalid_re() {
            let filter = FilterBuilder::new().instance_id("\\i-.*").build();

            assert!(&filter.is_err())
        }

        fn create_instance() -> InstanceDescriptor {
            let mut tags = HashMap::new();
            tags.insert("Name".to_owned(), Some("my_instance".to_owned()));
            tags.insert("Intent".to_owned(), Some("my_project".to_owned()));

            let instance = InstanceDescriptor {
                instance_id: Some("i-12345".to_owned()),
                image_id: Some("ami-12345".to_owned()),
                tags: Some(tags),
                ..Default::default()
            };
            instance
        }

        #[test]
        fn filter_instance_with_empty_filter() {
            let instance = create_instance();
            let filter = FilterBuilder::new()
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_true()
        }

        #[test]
        fn filter_instance_with_instance_id_okay() {
            let instance = create_instance();
            let filter = FilterBuilder::new()
                .instance_id("i-.*")
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_true()
        }

        #[test]
        fn filter_instance_with_instance_id_fail() {
            let instance = create_instance();
            let filter = FilterBuilder::new()
                .instance_id("instance-.*")
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_false()
        }

        #[test]
        fn filter_instance_with_instance_id_and_image_id_okay() {
            let instance = create_instance();
            let filter = FilterBuilder::new()
                .instance_id("i-.*")
                .image_id("ami-.*")
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_true()
        }

        #[test]
        fn filter_instance_with_instance_id_and_image_id_fail() {
            let instance = create_instance();
            let filter = FilterBuilder::new()
                .instance_id("i-.*")
                .image_id("image-.*")
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_false()
        }

        #[test]
        fn filter_instance_with_tags_okay() {
            let instance = create_instance();

            let mut tags = HashMap::new();
            tags.insert("Name".to_owned(), Some("my_.*"));
            tags.insert("Intent".to_owned(), Some("my_.*"));
            let filter = FilterBuilder::new()
                .tags(tags)
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_true()
        }

        #[test]
        fn filter_instance_with_tag_without_value_okay() {
            let instance = create_instance();

            let mut tags = HashMap::new();
            tags.insert("Name".to_owned(), None);
            let filter = FilterBuilder::new()
                .tags(tags)
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_true()
        }

        #[test]
        fn filter_instance_with_missing_tag() {
            let instance = create_instance();

            let mut tags = HashMap::new();
            tags.insert("NoSuchTagName".to_owned(), Some("my_.*"));
            let filter = FilterBuilder::new()
                .tags(tags)
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_false()
        }

        #[test]
        fn filter_instance_with_tags_fail() {
            let instance = create_instance();

            let mut tags = HashMap::new();
            tags.insert("Name".to_owned(), Some("my_.*"));
            tags.insert("Intent".to_owned(), Some("not_my_.*"));
            let filter = FilterBuilder::new()
                .tags(tags)
                .build()
                .expect("Failed to build filter");

            assert_that(&filter.filter(&instance)).is_false()
        }
    }
}
