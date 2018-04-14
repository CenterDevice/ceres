# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/lukaspustina/ceres/compare/v0.0.8...HEAD
[0.0.8]: https://github.com/lukaspustina/ceres/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/lukaspustina/ceres/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/lukaspustina/ceres/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/lukaspustina/ceres/compare/v0.0.4...v0.0.5

