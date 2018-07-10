# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Add `ceres stories prepare`
- Add `ceres stories start`


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

[Unreleased]: https://github.com/lukaspustina/ceres/compare/v0.0.18...HEAD
[0.0.18]: https://github.com/lukaspustina/ceres/compare/v0.0.17...v0.0.18
[0.0.17]: https://github.com/lukaspustina/ceres/compare/v0.0.16...v0.0.17
[0.0.16]: https://github.com/lukaspustina/ceres/compare/v0.0.15...v0.0.16
[0.0.15]: https://github.com/lukaspustina/ceres/compare/v0.0.14...v0.0.15
[0.0.14]: https://github.com/lukaspustina/ceres/compare/v0.0.13...v0.0.14
[0.0.13]: https://github.com/lukaspustina/ceres/compare/v0.0.12...v0.0.13
[0.0.12]: https://github.com/lukaspustina/ceres/compare/v0.0.11...v0.0.12
[0.0.11]: https://github.com/lukaspustina/ceres/compare/v0.0.10...v0.0.11
[0.0.10]: https://github.com/lukaspustina/ceres/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/lukaspustina/ceres/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/lukaspustina/ceres/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/lukaspustina/ceres/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/lukaspustina/ceres/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/lukaspustina/ceres/compare/v0.0.4...v0.0.5

