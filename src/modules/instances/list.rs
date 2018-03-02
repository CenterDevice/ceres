use clap::{App, Arg, ArgMatches, SubCommand};

use config::{Config, Provider};
use run_config::RunConfig;
use modules::*;
use output::OutputInstances;
use output::instances::{JsonOutputInstances, OutputType, TableOutputInstances};
use provider::{DescribeInstances, InstanceDescriptor, InstanceDescriptorFields};

pub const NAME: &str = "list";

pub struct List;

impl Module for List {
    fn build_sub_cli() -> App<'static, 'static> {
        SubCommand::with_name(NAME)
            .about("List instances")
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
    let instances = list_instances(args, run_config, config)?;
    let _ = output_instances(args, run_config, config, &instances)?;

    Ok(())
}

fn list_instances(
    _: &ArgMatches,
    run_config: &RunConfig,
    config: &Config,
) -> Result<Vec<InstanceDescriptor>> {
    let &Provider::Aws(ref provider) = match run_config.active_profile.as_ref() {
        "default" => config.get_default_provider(),
        s => config.get_provider_by_profile(s),
    }.chain_err(|| ErrorKind::ModuleFailed(NAME.to_owned()))?;

    provider
        .describe_instances()
        .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
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
            let output = TableOutputInstances {
                fields,
                tags_filter: Some(vec!["Name".to_owned(), "Intent".to_owned()]),
            };

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        }
        OutputType::Json => {
            let output = JsonOutputInstances;

            output
                .output(&mut stdout, instances)
                .chain_err(|| ErrorKind::ModuleFailed(String::from(NAME)))
        }
    }
}

mod filter {
    use regex::Regex;

    use std::collections::HashMap;
    use provider::InstanceDescriptor;

    macro_rules! filter_builder {
        ($($field:tt),+) => {
            struct FilterBuilder<'a> {
                $($field: Option<&'a str>),*,
                tags: Option<HashMap<String, Option<&'a str>>>
            }

            impl < 'a > FilterBuilder < 'a > {
                pub fn new() -> Self {
                    FilterBuilder {
                        $($field: None),*,
                        tags: None
                    }
                }

                $(
                pub fn $field(mut self, $field: &'a str) -> Self {
                    self.$field = Some($field);
                    self
                }
                )*

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
                        tags: if let Some(tags) = self.tags {
                                // TODO: Does not work with empty RE
                                let h = tags.into_iter().map(|(k, v)| (
                                    k, 
                                    if let Some(v) = v { Regex::new(v).ok() } else { None }
                                )).collect();
                                Some(h)
                            } else {
                                None
                            }
                    };

                    Ok(filter)
                }
            }

            struct Filter {
                $($field: Option<Regex>),*,
                tags: Option<HashMap<String, Option<Regex>>>
            }

           impl Filter {
                pub fn filter(&self, instance: &InstanceDescriptor) -> bool {
                    $(
                    if let Some(ref re) = self.$field {
                        if !re.is_match(instance.$field.as_ref().unwrap()) { return false };
                    }
                    )*

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

    error_chain! {
        errors {
            FilterRegexError(re: String, field: String) {
                description("Failed to build reg exp.")
                display("Failed to build reg exp '{}' for field '{}'.", re, field)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use spectral::prelude::*;
        use std::collections::HashMap;

        //--filter 'Instance=i-.*,Tags=Name=Packer.*:AnsibleHostGroup=batch_.*,State=stopped

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
