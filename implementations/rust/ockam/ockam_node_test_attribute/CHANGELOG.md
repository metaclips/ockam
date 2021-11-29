# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## unreleased

### Added

- Add git cliff and moved release.toml to root

### Removed

- Remove symlinks to `DEVELOP.md` and `LICENSE`


## v0.5.0 - 2021-11-22


### Changed

- Deny warnings in ci, not local development

### Fixed

- Fix a number of linter issues on the `develop` branch
- Enable ockam crate to use ockam_node_test_attribute


## v0.4.0 - 2021-11-15
### Changed
- fix unused variable warning in ockam_node_test_attribute
- Dependencies updated

## v0.3.0 - 2021-11-08
### Added
- add hygiene module
- add "no_main" feature to "node" macro
### Changed
- Dependencies updated
- node macro infers `Context` and `Result` types

## v0.2.0 - 2021-11-01
### Changed
- Dependencies updated

## v0.1.0 - 2021-10-16

Initial release.

### Added
- `node` - a proc macro that defines a custom `#[node]` attribute.
- `node_test` - a proc macro that defines a custom `#[node_test]` attribute.
