# NAME

CenterDevice SRE (ceres) -- Ceres the goddess of agriculture, grain crops, fertility and motherly relationships.


# SYNOPSIS

ceres [*options*] *MODULE*

mhost --help

mhost --version


# DESCRIPTION

ceres is a CLI tool for common SRE and ops tasks for CenterDevice.

ceres comes with different modules. It supports human readable as well as JSON output for post-processing with other tools like `jq`. 

For ceres to work properly -- actually to work at all -- a configuration file is required that specifies the stage and its endpoints as well as required credentials for CenterDevice environments. See *ceres.conf* for details.

The project home page currently *https://github.com/lukaspustina/ceres*.


# COMMON OPTIONS

--config *config*
: Sets config file to use. Default is *~/.ceres.conf*

--profile *profile*
: Sets profile to use. Default it to use the *default* profile.

--help
: Prints help information


# LESS COMMON OPTIONS

-V, --version
: Prints version information.


# MODULES

Currently, there is only one module called *instances*.


## INSTANCES 

The *instances* modules interacts with instances in the environment selected by the profile to use. Currently the only command supported is *list*.

### instances list [*options*]

  *instances list* shows all currently active instances. Instances can be filtered and the output can be controlled to allow for human readable format or JSON format for post-processing.

  -f, --filter *filter*
  : Filters instances by description fields. The filter syntax is *\<description field\>=\<reg ex\>*. Multiple filters can be used and have to be separated by ','. Each description field will be matched against the regular expression. Only instances matching all description field will be selected.

  The special description field *Tags* supports a specialized syntax which is *Tags=\<tag name\>[=\<reg ex\>]*. Multiple tags can be used and have to be separated by ':'. If a tag is specified without a regular expressions, only instances bearing that tag will be selected. If a tag is specified with a regular expression, only instances bearing that tag with a matching value will be selected. Instances have to match all tags to be selected.

  For example, the filter 'InstanceId=i-.\*,Tags=Name:AnsibleHostGroup=batch_.\*,State=stopped' will only selected instances with an instance id beginning in 'i-', the tag 'Name' set, the tag 'AnsibleHostGroup' with a value starting in 'batch_' and in the state 'stopped' will be selected. 

  The available description field to filter against are:

    BlockDeviceMappings, Hypervisor, IamInstanceProfile, ImageId, InstanceId, InstanceType, LaunchTime, Monitoring, Placement, PrivateDnsName, PrivateIpAddress, PublicDnsName, PublicIpAddress, RootDeviceName, RootDeviceType, SecurityGroups, State, StateReason, Tags(_), VirtualizationType, VpcId

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human* and *json*.

  --output-options *output-options*
  : Selects the instance description fields for human output. The default is 'InstanceId,InstanceType,State,PrivateIpAddress,PublicIpAddress,LaunchTime'. The special description field *Tags* may take a list of concrete tags to show. The corresponding syntax is similar to the tags filter and is *Tags[=\<tag name\>]*. Multiple tags can be used have to separated by ':'.

  For example, the output options 'InstanceId,Tags=Name:AnsibleHostGroup' outputs the instance id and the tags 'Name' and AnsibleHostGroup' for all selected instances.

  The available options are: 

    BlockDeviceMappings, Hypervisor, IamInstanceProfile, ImageId, InstanceId, InstanceType, LaunchTime, Monitoring, Placement, PrivateDnsName, PrivateIpAddress, PublicDnsName, PublicIpAddress, RootDeviceName, RootDeviceType, SecurityGroups, State, StateReason, Tags(_), VirtualizationType, VpcId

### instances ssh [*options*] *INSTANCE_ID* [-- *COMMAND_ARGS ...*]

  *instances ssh* connects to an instance and either opens an interactive shell or runs a single command. By default, the instance' private IP address is used. The remote login name is read from the corresponding profile configuration in the configuration file, or set as option, or the local user name is used.

  *INSTANCE_ID*
  : Sets the instance id to connect to.

  *COMMAND_ARGS ...*
  : Sets the command and its arguments to execute on the remote instance. These have to be that last argument which requires a prefixing *--*.

  -l, --login-name *login-name*
  : Sets remote login name

  -p, --public-ip
  : Use public IP address of instance

  --ssh-opt *ssh-opts* ...
  : Passes an option to ssh. This may be used multiple times.

### instances terminate [*options*] *INSTANCE_ID ...*

  *instances terminate* terminates instances by instance id and outputs the corresponding state changes. A prompt will ask for confirmation before any termination is executed. The output can be controlled to allow for human readable format or JSON format for post-processing.

  *INSTANCE_ID ...*
  : Sets the instance id to terminate. Multiple instance ids may be set.

  -d, --dry
  : Activates dry run. Permissions and instance ids will be checked by AWS, but no instance will be terminated.

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human* and *json*.

  --yes-i-really-really-mean-it
  : Don't ask for confirmation and terminate instances immediately.

# SHELL COMPLETION

completions --shell *shell*
: Generates shell completions for supported shells which are currently bash, fish, and zsh.


# FILES
  *~/.ceres.conf*


# SEE ALSO
  ceres.conf(5)

# COPYRIGHT AND LICENSE

Copyright (c) 2018 Lukas Pustina. Licensed under the MIT License. See *https://github.com/lukaspustina/ceres/blob/master/LICENSE* for details.

