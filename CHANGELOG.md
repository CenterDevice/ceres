# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.30] - 2019-10-02

### Fixes
* [#125](https://github.com/CenterDevice/ceres/issues/125): Fehlerhafte Ceres Dokumentation für den CenterDevice Upload 
* [#126](https://github.com/CenterDevice/ceres/issues/126): Ceres CenterDevice Aufrufe sollten JSON als Output zurückgeben

## [0.0.29] - 2019-10-01

### Fixes
* Bug in handling of attachment-only comments 

## [0.0.28] - 2019-09-30

### Added
* Pivotal Tracker ticket export for markdown and JSON

## [0.0.27] - 2019-06-21

### Added
* -i option to cssh to allow connecting by instance id
* centerdevice users
* centerdevice collections
* -R for centerdevice users and collections to resolves ids to human readable names

### Changed
* centerdevice search shows number of hits

### Fixed
* Fixes some documentation and help text mistakes


## [0.0.26] - 2019-06-13

### Add
* Basic CenterDevice client for auth, upload, download, delete, and search.

## [0.0.24] - 2018-10-24
- Add optional 'token' parameter to AWS provider

## [0.0.23] - 2018-08-20
- Update health check to understand 'global' key

## [0.0.22] - 2018-09-18
- Fix `flatten()` issue introduced by Rust 1.29

## [0.0.21] - 2018-09-04

- Add filter to cssh so it only considers running instances
- Update pivotal tracker tasks
- Update dependencies


## [0.0.20] - 2018-07-13

- Add `ceres health check`


## [0.0.19] - 2018-07-10

### Added
- Add `ceres stories prepare`
- Add `ceres stories start`
- Add `contrib/cssh`


## [0.0.18] - 2018-06-23

### Added
- Add `ceres statuspage show`

### Fixed
- Fix #46: `instance list --output-options Tags` panics if no tags are set for an instance


## [0.0.17] - 2018-06-18

### Added
- Add `ceres infrastructure images [list|build]`
- Add `ceres infrastructure resources [list|build]`


## [0.0.16] - 2018-06-18

### Added
- Add `ceres infrastructure asp list`
- Add `ceres infrastructure asp build`


## [0.0.15] - 2018-06-01

### Added
- Add `ceres ops webserver backup`
- Add `ceres ops asp run`

## [0.0.14] - 2018-05-28

### Fixed
- Fixes #3: `ceres ops issues create` wait for with prompt in order to ensure that your editor finished unless `--no-wait` is set.
- Fixes #25: "_Allow ceres instances ssh to receive Instance ID from stdin_" by adding plain output to `instances list`.


## [0.0.13] - 2018-05-17

### Fixed
- Fixes #14 by treating `ssh-opts` the same for `instances run` and `instances ssh`. You have to use it like this:
    ```bash
    ceres instances ssh i-0af7400f10e5b0249 --ssh-opt=-i --ssh-opt=/Users/lukas/.ssh/id_rsa
    ```


## [0.0.12] - 2018-05-17

### Added
- instance commands take instance id via stdin where appropriate. So you can do stuff like this:
    ```bash
    ceres instances list --filter "Tags=Name=.*rabbit.*" --output json | ceres instances run - -- w
    ```


## [0.0.11] - 2018-05-16

### Added
- Move Rust stable to required build target

### Fixed
- Fix zsh completions


## [0.0.10] - 2018-05-15

### Added
- Starting and stopping of instances.
- Add `human_panic`.

### Fixed
- Fix Homebrew recipe.


## [0.0.9] - 2018-04-15

Add deployments in Travis build.


## [0.0.8] - 2018-04-14

### Added
- Arg `no-color` to turn off colorful output -- helpful for non-tty usage.
- Use `smart_load` from `clams` to load configuration.
- Use `clams::prelude`.
- Add `--browser` option to `ops issues create` that opens a new issue in your default browser

### Changed
- Config format, `issue_tracker` section to accommodate `--browser` options. This is a **breaking** change.

### Removed
- Broken use case tests


## [0.0.7] - 2018-04-06

### Add
- show-example-config module to echo an example configuration.


## [0.0.6] - 2018-04-01

### Changed
- Use utils from `clams` instead of reinventing the wheel and maintaining all the general purpose utils.


## [0.0.5] - 2018-03-29

### Added
- `consul list` module: List nodes from consul cluster filtered by service names and service tags.
- plain output variant: Currently only available for consul list

### Changed
- `instances terminate` uses `warn!` macro instead of yellow `println!` to output warning in case of active dry mode.

[Unreleased]: https://github.com/centerdevice/ceres/compare/v0.0.30...HEAD
[0.0.30]: https://github.com/centerdevice/ceres/compare/v0.0.29...v0.0.30
[0.0.29]: https://github.com/centerdevice/ceres/compare/v0.0.28...v0.0.29
[0.0.28]: https://github.com/centerdevice/ceres/compare/v0.0.27...v0.0.28
[0.0.27]: https://github.com/centerdevice/ceres/compare/v0.0.25...v0.0.27
[0.0.26]: https://github.com/centerdevice/ceres/compare/v0.0.24...v0.0.26
[0.0.24]: https://github.com/centerdevice/ceres/compare/v0.0.23...v0.0.24
[0.0.23]: https://github.com/centerdevice/ceres/compare/v0.0.22...v0.0.23
[0.0.22]: https://github.com/centerdevice/ceres/compare/v0.0.21...v0.0.22
[0.0.21]: https://github.com/centerdevice/ceres/compare/v0.0.20...v0.0.21
[0.0.20]: https://github.com/centerdevice/ceres/compare/v0.0.19...v0.0.20
[0.0.19]: https://github.com/centerdevice/ceres/compare/v0.0.18...v0.0.19
[0.0.18]: https://github.com/centerdevice/ceres/compare/v0.0.17...v0.0.18
[0.0.17]: https://github.com/centerdevice/ceres/compare/v0.0.16...v0.0.17
[0.0.16]: https://github.com/centerdevice/ceres/compare/v0.0.15...v0.0.16
[0.0.15]: https://github.com/centerdevice/ceres/compare/v0.0.14...v0.0.15
[0.0.14]: https://github.com/centerdevice/ceres/compare/v0.0.13...v0.0.14
[0.0.13]: https://github.com/centerdevice/ceres/compare/v0.0.12...v0.0.13
[0.0.12]: https://github.com/centerdevice/ceres/compare/v0.0.11...v0.0.12
[0.0.11]: https://github.com/centerdevice/ceres/compare/v0.0.10...v0.0.11
[0.0.10]: https://github.com/centerdevice/ceres/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/centerdevice/ceres/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/centerdevice/ceres/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/centerdevice/ceres/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/centerdevice/ceres/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/centerdevice/ceres/compare/v0.0.4...v0.0.5

