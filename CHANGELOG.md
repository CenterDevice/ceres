# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5] - 2018-03-29

### Added
- `consul list` module: List nodes from consul cluster filtered by service names and service tags.
- plain output variant: Currently only available for consul list

### Changed
- `instances terminate` uses `warn!` macro instead of yellow `println!` to output warning in case of active dry mode.

