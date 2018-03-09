# NAME

ceres.conf - configures profiles and others settings for ceres. This configuration file uses TOML syntax. It may define multiple profiles with different names.

# DESCRIPTION

*default_profile* = "\<profile name\>"

[logging]

default = "warn"

ceres = "info"

[profiles."\<profile name\>"]

ssh_user = "\<a user name\>"

[profiles."\<profile name\>".provider]

*type* = "aws"

*access_key_id* = "\<access key id\>"

*secret_access_key* = "\<secret access key\>"

*region* = "\<AWS region string\>"

*role_arn* = "\<role arn/>"


# SEE ALSO
  ceres(1)

# COPYRIGHT AND LICENSE

Copyright (c) 2018 Lukas Pustina. Licensed under the MIT License. See *https://github.com/lukaspustina/ceres/blob/master/LICENSE* for details.

