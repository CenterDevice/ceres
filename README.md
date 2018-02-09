# CEnteRdEvice Sre (ceres)

Ceres the goddess of agriculture, grain crops, fertility and motherly relationships.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

- [General Requirements](#general-requirements)
- [Use Cases](#use-cases)
  - [PoC: List AWS EC2 instances in one account using "Assume Role" and filter by tags](#poc-list-aws-ec2-instances-in-one-account-using-assume-role-and-filter-by-tags)
    - [Requirements](#requirements)
- [Todos](#todos)
- [Pointer](#pointer)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## General Requirements

* Dynamic completion in zsh for instances etc.

* maintain man page


## Use Cases

### PoC: List AWS EC2 instances in one account using "Assume Role" and filter by tags

```bash
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
ceres --profile prod@aws instances list --filter 'adlkfjdafjdsf'
```

#### Requirements

* [ ] Use Assume Role mechanism

* [ ] Show usage of clippy with subcommands

* [ ] Show dynamic completion in zsh


## Todos

## Pointer
* https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md
* https://github.com/cmsd2/stscli

