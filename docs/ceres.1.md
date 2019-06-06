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

The project home page currently *https://github.com/centerdevice/ceres*.


# COMMON OPTIONS

--config *config*
: Sets config file to use. Default is *~/.ceres.conf*

--no-color
: Turns off colorful output. Helpful for non-tty usage.

--profile *profile*
: Sets profile to use. Default it to use the *default* profile.

--help
: Prints help information


# LESS COMMON OPTIONS

-V, --version
: Prints version information.


# MODULES


## CENTERDEVICE

The *centerdevice* module interacts with a CenterDevice instance and offers basic primitives.

### centerdevice auth [*options*]

  *centerdevice auth* authenticates against a particular CenterDevice instance.

  -s, --show
  : on successful authentication, ceres prints the received token to stdout.

  -S, --save
  : on successful authentication, ceres saves the token to its configuration file.

  Attention: This will overwrite the entire configuration file and thus, comments will be lost and any order of elements will change.


## CONSUL

The *consul* module interacts with the consul cluster in the environment selected by the profile to use.

### consul list [*options*]

  *consul list* shows all currently active nodes and their services. Nodes can be filtered by service names and service tags. The output can be controlled to allow for human readable and plain format as well as plain or JSON format for post-processing.

  -o, --output *output*
  : Selects output format. The default is *human* and the possible values are: 
    
    human, json, plain

  --output-options *output-options*
  : Selects the nodes description fields for human and plain output. The special description field *MetaData* may take a list of concrete meta data tags to show. The corresponding syntax is the same as for *instance list* output option's Tag. There is a shortcut to select all fields by using the field *all*.

  The available description fields are:

    Id, Name, MetaData(_), Address, ServicePort, ServiceTags, ServiceId, ServiceName, Healthy

  -s, --services *services*...
  : Filters services for specific service names.

  -t, --tags *tags*...
  : Filters services for specific tags.


## HEALTH

The *health* module interacts with the health check resources of a CenterDevice instance configured per profile.

### health check [*options*]

  *health check* queries health checks for all resources, i.e., "admin", "api", "app", "auth", "public", "sales", "upload".

  -o, --output *output*
  : Selects output format [default: human]  [possible values: human, json, plain]


## INFRASTRUCTURE

The *infrastructure* modules automate building, planning, and deploying infrastructure as code resources from the CenterDevice *infrastructure* repository.

### infrastructure asp list [*options*]

  *infrastructure asp list* identifies all ansible setup packages (ASPs) in a given sub-directory of the CenterDevice *infrastructure* repository specified either in the ceres configuration file or passed via command line.

  --base-dir *base-dir*
  : Overwrites base dir from ceres configuration file

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.


### infrastructure asp build [*options*] -p *project* -r *resource*

  *infrastructure asp build* builds a specific ansible setup packages (ASP) in a given sub-directory of the CenterDevice *infrastructure* repository specified either in the ceres configuration file or passed via command line and, on success uploads the ASP to S3.

  -p, --project *project*
  : Sets project

  -r, --resource *resource*
  : Sets resource to build

  --no-progress-bar
  : Do not show progress bar during command execution. This is useful for non-interactive sessions.

  --show-all
  : Show all command results. By default show only results of failed commands.

  --base-dir *base-dir*
  : Overwrites base dir from ceres configuration file

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.

  --timeout *timeout*
  : Sets the timeout in sec for command to finish. Default is 300 sec.


### infrastructure images list [*options*]

  *infrastructure images list* identifies all images in a given sub-directory of the CenterDevice *infrastructure* repository specified either in the ceres configuration file or passed via command line.

  --base-dir *base-dir*
  : Overwrites base dir from ceres configuration file

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.


### infrastructure images build [*options*] -p *project* -r *resource*

  *infrastructure images build* builds a specific image in a given sub-directory of the CenterDevice *infrastructure* repository specified either in the ceres configuration file or passed via command line.

  -p, --project *project*
  : Sets project

  -r, --resource *resource*
  : Sets resource to build

  --no-progress-bar
  : Do not show progress bar during command execution. This is useful for non-interactive sessions.

  --show-all
  : Show all command results. By default show only results of failed commands.

  --base-dir *base-dir*
  : Overwrites base dir from ceres configuration file

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.

  --timeout *timeout*
  : Sets the timeout in sec for command to finish. Default is 300 sec.


### infrastructure resources list [*options*]

  *infrastructure resources list* identifies all resources in a given sub-directory of the CenterDevice *infrastructure* repository specified either in the ceres configuration file or passed via command line.

  --base-dir *base-dir*
  : Overwrites base dir from ceres configuration file

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.


### infrastructure resources build [*options*] -p *project* -r *resource*

  *infrastructure resources build* builds a specific resources in a given sub-directory of the CenterDevice *infrastructure* repository specified either in the ceres configuration file or passed via command line.

  -p, --project *project*
  : Sets project

  -r, --resource *resource*
  : Sets resource to build

  --no-progress-bar
  : Do not show progress bar during command execution. This is useful for non-interactive sessions.

  --show-all
  : Show all command results. By default show only results of failed commands.

  --base-dir *base-dir*
  : Overwrites base dir from ceres configuration file

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.

  --timeout *timeout*
  : Sets the timeout in sec for command to finish. Default is 300 sec.


## INSTANCES 

The *instances* modules interacts with instances in the environment selected by the profile to use.

### instances list [*options*]

  *instances list* shows all currently active instances. Instances can be filtered and the output can be controlled to allow for human readable, plain, or JSON format for post-processing.

  -f, --filter *filter*
  : Filters instances by description fields. The filter syntax is *\<description field\>=\<reg ex\>*. Multiple filters can be used and have to be separated by ','. Each description field will be matched against the regular expression. Only instances matching all description field will be selected.

  The special description field *Tags* supports a specialized syntax which is *Tags=\<tag name\>[=\<reg ex\>]*. Multiple tags can be used and have to be separated by ':'. If a tag is specified without a regular expressions, only instances bearing that tag will be selected. If a tag is specified with a regular expression, only instances bearing that tag with a matching value will be selected. Instances have to match all tags to be selected.

  For example, the filter 'InstanceId=i-.\*,Tags=Name:AnsibleHostGroup=batch_.\*,State=stopped' will only selected instances with an instance id beginning in 'i-', the tag 'Name' set, the tag 'AnsibleHostGroup' with a value starting in 'batch_' and in the state 'stopped' will be selected. 

  The available description field to filter against are:

    BlockDeviceMappings, Hypervisor, IamInstanceProfile, ImageId, InstanceId, InstanceType, LaunchTime, Monitoring, Placement, PrivateDnsName, PrivateIpAddress, PublicDnsName, PublicIpAddress, RootDeviceName, RootDeviceType, SecurityGroups, State, StateReason, Tags(_), VirtualizationType, VpcId

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human*, *plain*, and *json*.

  --output-options *output-options*
  : Selects the instance description fields for human output. The default is 'InstanceId,InstanceType,State,PrivateIpAddress,PublicIpAddress,LaunchTime'. The special description field *Tags* may take a list of concrete tags to show. The corresponding syntax is similar to the tags filter and is *Tags[=\<tag name\>]*. Multiple tags can be used have to separated by ':'.

  For example, the output options 'InstanceId,Tags=Name:AnsibleHostGroup' outputs the instance id and the tags 'Name' and AnsibleHostGroup' for all selected instances.

  The available options are: 

    BlockDeviceMappings, Hypervisor, IamInstanceProfile, ImageId, InstanceId, InstanceType, LaunchTime, Monitoring, Placement, PrivateDnsName, PrivateIpAddress, PublicDnsName, PublicIpAddress, RootDeviceName, RootDeviceType, SecurityGroups, State, StateReason, Tags(_), VirtualizationType, VpcId

### instances run [*options*] *INSTANCE_ID* ... [-- *COMMAND_ARGS ...*]

  *instances run* connects to multiple instance and runs a single command on each instance. By default, the instances' private IP addresses are used. The remote login name is read from the corresponding profile configuration in the configuration file, or set as option, or the local user name is used. The difference of this command compared to *instances ssh* is that this command logs all output to separate files instead of printing to all output to the console.

  *INSTANCE_ID ...*
  : Sets the instance ids to connect to; or '-' to read json with instance ids from stdin. Multiple instance ids may be set.

  *COMMAND_ARGS ...*
  : Sets the command and its arguments to execute on the remote instance. These have to be that last argument which requires a prefixing *--*.

  -l, --login-name *login-name*
  : Sets remote login name

  --no-progress-bar
  : Do not show progress bar during command execution. This is useful for non-interactive sessions.

  -p, --public-ip
  : Use public IP address of instance

  --show-all
  : Show all command results. By default show only results of failed commands.

  --ssh-opt *ssh-opts* ...
  : Passes an option to ssh. This may be used multiple times.

  --timeout *timeout*
  : Sets the timeout in sec for command to finish. Default is 300 sec.

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

### instances start [*options*] *INSTANCE_ID ...*

  *instances start* starts instances by instance id and outputs the corresponding state changes. The output can be controlled to allow for human readable format or JSON format for post-processing.

  *INSTANCE_ID ...*
  : Sets the instance id to start; or '-' to read json with instance ids from stdin. Multiple instance ids may be set.

  -d, --dry
  : Activates dry run. Permissions and instance ids will be checked by AWS, but no instance will be started.

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human* and *json*.

### instances stop [*options*] *INSTANCE_ID ...*

  *instances stop* stops instances by instance id and outputs the corresponding state changes. A prompt will ask for confirmation before any instance is stopped. The output can be controlled to allow for human readable format or JSON format for post-processing.

  *INSTANCE_ID ...*
  : Sets the instance id to stop; or '-' to read json with instance ids from stdin. Multiple instance ids may be set.

  -d, --dry
  : Activates dry run. Permissions and instance ids will be checked by AWS, but no instance will be stopped.

  --force
  : Forces instances to stop. The instances do not have an opportunity to flush file system caches or file system metadata. If you use this option, you must perform file system check and repair procedures. 

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human* and *json*.

  --yes-i-really-really-mean-it
  : Don't ask for confirmation and stop instances immediately.

### instances terminate [*options*] *INSTANCE_ID ...*

  *instances terminate* terminates instances by instance id and outputs the corresponding state changes. A prompt will ask for confirmation before any termination is executed. The output can be controlled to allow for human readable format or JSON format for post-processing.

  *INSTANCE_ID ...*
  : Sets the instance id to terminate; or '-' to read json with instance ids from stdin. Multiple instance ids may be set.

  -d, --dry
  : Activates dry run. Permissions and instance ids will be checked by AWS, but no instance will be terminated.

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human* and *json*.

  --yes-i-really-really-mean-it
  : Don't ask for confirmation and terminate instances immediately.

## OPS

The *ops* modules include various ops related commands to ease regular ops tasks.


### ops asp run [*options*]

  *ops asp run* run ASP on multiple instances. By default, the instances' private IP addresses are used. The remote login name is read from the corresponding profile configuration in the configuration file, or set as option, or the local user name is used. 

  -l, --login-name *login-name*
  : Sets remote login name

  --no-progress-bar
  : Do not show progress bar during command execution. This is useful for non-interactive sessions.

  -p, --public-ip
  : Use public IP address of instance

  --show-all
  : Show all command results. By default show only results of failed commands.

  --ssh-opt *ssh-opts* ...
  : Passes an option to ssh. This may be used multiple times.

  --timeout *timeout*
  : Sets the timeout in sec for command to finish. Default is 300 sec.

### ops issues browse [*options*]

  *ops issues browse* opens the GitHub ops issues your default web browser.

  -p, --project
  : Opens the corresponding ops issues project instead of the issues list.


### ops issues create [*options*]

  *ops issues create* creates a new ops issue either from a file or using your default *$EDITOR* pre-filled from a template.

  --browser
  : Opens new issue in default browser with *template* from config setting or set via *--template*. This setting conflicts with *-f* and *-i*.

  -i, --interactive
  : Opens $EDITOR to write issue contents using *template* from config setting or set via *--template*. This setting conflicts with *--browser* and *-f*.

  --show-in-browser
  : Opens newly created issue in web browser.

  --no-wait
  : Do not wait for editor to finish in interactive mode

  -f, --filename *filename*
  : Sets file name of markdown file to fill issue with. This option conflicts with *-i*.

  -l, --label *label* ...
  : Sets labels for new issue.

  --template *template*
  : Uses this template to pre-fill editor; defaults to config setting. This option conflicts with *-f*.

  -t, --title *title*
  : Sets title for issue.


### ops webserver backup [*options*]

  *ops webserver backup* executes the backup scripts on the webserver. All machines with the tag "Intent=webserver" are considered webservers. By default, the instances' private IP addresses are used. The remote login name is read from the corresponding profile configuration in the configuration file, or set as option, or the local user name is used. This command assumes that there is only one webserver and refuses to execute if more than one webservers are found. This can be overpowered using `--force`

  -l, --login-name *login-name*
  : Sets remote login name

  --no-progress-bar
  : Do not show progress bar during command execution. This is useful for non-interactive sessions.

  --force
  : Force execution even if more than one webservers are found. Use this with caution.

  -p, --public-ip
  : Use public IP address of instance

  --show-all
  : Show all command results. By default show only results of failed commands.

  --ssh-opt *ssh-opts* ...
  : Passes an option to ssh. This may be used multiple times.

  --timeout *timeout*
  : Sets the timeout in sec for command to finish. Default is 300 sec.


## STATUSPAGES

The *statuspages* modules interacts with the statuspage.io status pages.

### statuspages show [*options*]

  *statuspages show* show the current status for every statuspage.

  -o, --output *output*
  : Selects output format. The default is *human*. Available options are *human* and *json*


## STORIES

The *stories* modules interacts with the story trackers, i.e., currently PivotalTracker.

### stories prepare [*options*] *STORY_ID*

  *stories prepare* prepares a story. Currently, the 13 steps from the infrastructure story process are added as tasks. These tasks are only added, if the story does not have any other tasks. This behavior can be change with the *--force* flag.

  *STORY_ID*
  : The id of the story to prepare. The id may start with a '#' the same way, PivotalTracker uses ids. If used with '#', then the id needs to be surrounded by tickets to allow for shell escaping, e.g., '#12345'.

  --force
  : Forces creation of tasks even when other tasks already exist.


### stories start [*options*] *STORY_ID*

  *stories start* starts a story. A story will only be started, if it is currently in the 'unstarted' state. This behavior can be change with the *--force* flag. A story can only be started if already estimated.

  *STORY_ID*
  : The id of the story to start. The id may start with a '#' the same way, PivotalTracker uses ids. If used with '#', then the id needs to be surrounded by tickets to allow for shell escaping, e.g., '#12345'.

  --force
  : Sets state to started even if current state is not 'unstarted'.


# SHELL COMPLETION

completions --shell *shell*
: Generates shell completions for supported shells which are currently bash, fish, and zsh.


# SHOW EXAMPLE CONFIGURATION

show-example-config
: Show an example configuration file which can be used as a template to crate a working configuration file.


# FILES
  *~/.ceres.conf*


# SEE ALSO
  ceres.conf(5)

# COPYRIGHT AND LICENSE

Copyright (c) CenterDevice. Licensed under the MIT License. See *https://github.com/centerdevice/ceres/blob/master/LICENSE* for details.

