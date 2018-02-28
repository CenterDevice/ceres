# CEnteRdEvice Sre (ceres)

Ceres the goddess of agriculture, grain crops, fertility and motherly relationships.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

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

## General Requirements

* Dynamic completion in zsh for instances etc.

* maintain man page


## Use Cases

### PoC: List AWS EC2 instances in one account using "Assume Role" and filter by tags

```bash
ceres --profile prod@aws instances list --filter 'Tag:Value,Tag2:Value2' -o [humon|json] --o-opts=instance_id,image_id,instance_type
```

#### Step 1

##### Requirements

* [X] Use Assume Role mechanism

* [X] Use profiles to select AWS account, Bosun and Consul etc. end-points, ssh keys

* [X] Read profile from configuration file: ~/.ceres.conf

* [X] Nice human readable output

* [X] Allow for selection of limited set of instance information for human display

* [ ] JSON output of all instance information

* [ ] Allow for filtering re/ Tags and other information with reg ex

##### Non-Functional Requirements

* [ ] brew installation

#### Step 2

##### Non-Functional Requirements

* [ ] Write man page

* [ ] Provider Abstraction -- abstract from rusoto

* [ ] Prepare for modules, sub-modules etc.


## Pointer
* https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md
* https://github.com/cmsd2/stscli

