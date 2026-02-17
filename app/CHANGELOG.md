# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/drewzemke/post-push-party/compare/v0.1.2...v0.1.3) - 2026-02-17

### Added

- *(cli)* change `status` subcommand to `points`
- *(cli)* add stats subcommand
- *(party)* simplify points breakdown if no commits counted
- *(party)* add color to points breakdown party
- *(party)* add fireworks party

### Fixed

- *(party)* text alignment in stats party
- *(party)* don't print extra blank line after fireworks party

### Other

- *(party)* extract utility functions for color/font control
- specificy readme in main package

## [0.1.2](https://github.com/drewzemke/post-push-party/compare/v0.1.1...v0.1.2) - 2026-02-15

### Other

- fix some lints
- add fireworks crate (unintegrated)
- convert to cargo workspace
