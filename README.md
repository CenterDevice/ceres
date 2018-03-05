# CEnteRdEvice Sre (ceres)

Ceres the goddess of agriculture, grain crops, fertility and motherly relationships.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [Installation](#installation)
- [Configuration](#configuration)
- [General Requirements](#general-requirements)
- [Use Cases](#use-cases)
  - [PoC: List AWS EC2 instances in one account using "Assume Role" and filter by tags](#poc-list-aws-ec2-instances-in-one-account-using-assume-role-and-filter-by-tags)
    - [Step 1](#step-1)
      - [Requirements](#requirements)
      - [Non-Functional Requirements](#non-functional-requirements)
    - [Step 2](#step-2)
      - [Non-Functional Requirements](#non-functional-requirements-1)
- [Pointer](#pointer)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Installation

`brew install lukaspustina/os/ceres --HEAD`


## Configuration

`ceres` tries to read `~/.ceres.conf` by default. See [example](examples/ceres.conf) for an example configuration.

## General Requirements

* Dynamic completion in zsh for instances etc.

* maintain man page


## Use Cases

### List AWS EC2 instances in one account using "Assume Role" and filter by tags

```bash
ceres --config ~/.ceres.conf --profile staging@cd instances list -o [humon|json] --output-options=InstanceId,Tags=Name:AnsibleHostGroup,State --filter 'Instance=i-.*,Tags=Name:AnsibleHostGroup=batch_.*,State=stopped'
```

