# CEnteRdEvice Sre (ceres)

Ceres the goddess of agriculture, grain crops, fertility and motherly relationships.

[![Linux and macOS Build Status](https://travis-ci.org/lukaspustina/ceres.svg?branch=master)](https://travis-ci.org/lukaspustina/ceres) [![codecov](https://codecov.io/gh/lukaspustina/ceres/branch/master/graph/badge.svg)](https://codecov.io/gh/lukaspustina/ceres) [![GitHub release](https://img.shields.io/github/release/lukaspustina/ceres.svg)](https://github.com/lukaspustina/ceres/releases) [![](https://img.shields.io/crates/v/ceres.svg)](https://crates.io/crates/ceres) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg?label=License)](./LICENSE)

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [Installation](#installation)
  - [Brew](#brew)
  - [Cargo](#cargo)
  - [From Source](#from-source)
- [Configuration](#configuration)
- [Use Cases](#use-cases)
  - [List AWS EC2 instances in one account using "Assume Role" and filter by tags](#list-aws-ec2-instances-in-one-account-using-assume-role-and-filter-by-tags)
- [Todos](#todos)
  - [CI/CD](#cicd)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Installation

### Brew

`brew install lukaspustina/os/ceres --HEAD`

### Cargo

`cargo install ceres --git https://github.com/lukaspustina/ceres`

### From Source

```
git clone https://github.com/lukaspustina/ceres
cd ceres
cargo install
```


## Configuration

`ceres` requires a configuration file in order to load profiles. By default, `ceres` tries to read `~/.ceres.conf`. See [example](examples/ceres.conf) for an example configuration.


## Use Cases

### List AWS EC2 instances in one account using "Assume Role" and filter by tags

```bash
ceres --config ~/.ceres.conf --profile staging@cd instances list -o [humon|json] --output-options=InstanceId,Tags=Name:AnsibleHostGroup,State --filter 'Instance=i-.*,Tags=Name:AnsibleHostGroup=batch_.*,State=stopped'
```

## Todos

### CI/CD

* [X] Travis build
* [X] Badges
  * [X] Build
  * [X] Test Coverage
* [X] brew?

