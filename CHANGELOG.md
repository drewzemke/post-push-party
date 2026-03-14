# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/drewzemke/post-push-party/compare/v0.1.4...v0.1.5) - 2026-03-14

### Added

- *(tui)* adjust layout, improve enabled/disabled indication, fix truncated descriptions
- *(tui)* hide palette config for parties that don't support color
- *(packs)* add back pack functionality

### Other

- *(tui)* use `Rect::layout` instead of `Layout::split` (in some but not all places)
- *(storage)* replace `BranchRefs::get_refs_for_repo` with a more targeted `get_ref` function
- *(stats)* compute stats via more efficient queries
- *(storage)* add `PushHistory::count_since` and use it in bonus track calculations
- *(storage)* internal refactor and slight API improvement for `PushHistory`
- remove serde-related stuff
- remove all old-format bincode-related stuff

## [0.1.4](https://github.com/drewzemke/post-push-party/compare/v0.1.3...v0.1.4) - 2026-03-12

### Added

- *(init)* specific error message when alias is already installed
- *(init)* drop custom jj hook impl and use ref-transaction git hook for jj :)
- *(storage)* patch ids are store/retrieved from sqlite
- *(storage)* push history reads/writes to/from sql instead of file
- *(storage)* store/access branch refs in sqlite db
- *(storage)* implement save and load to sqlite for State
- *(storage)* add storage module with sqlite connection and initial migration
- *(packs)* update points award by packs
- *(pack)* add premium pack
- *(packs)* set prices for basic pack
- *(pack)* first pass at pack-opening algorithm
- *(tui)* show number of unopened packs in tab header
- *(tui)* open packs in the packs panel
- *(store)* buy packs in the store
- *(pack)* get packs based on lifetime point accrual
- *(state)* add packs
- *(tui)* palette selector UI and code organization improvements
- *(tui)* select palettes in party config

### Fixed

- clippy warning for dev-only function
- *(storage)* use transaction handle when saving state
- *(pack)* points earned through packs don't count for lifetime points
- *(tui)* fix scrolling in store and party lists
- *(tui)* shimmer boarder shimmers less frequently, only uses yellow and white
- *(bonus)* "sniper" bonus track awards 1- or 2-line commits
- *(party)* use "point"/"points" correctly in base party

### Other

- some extra cleanup comments
- revert most of the last three commits but improve jj config management
- update readme to reflect jj hook changes
- *(cli)* remove `snapshot` functionality
- *(storage)* write logs to new location
- *(storage)* move `PushEntry` into same module as `PushHistory`
- *(storage)* extract state storage code to module
- *(deps)* add `rusqlite`
- *(state)* comment out stuff to get state to be bincode-compatible with v0.1.3
- add another very-vibecoded tool to help tune rarity probabilities
- *(tui)* extract `PaletteSelector` widget
- *(tui)* wip of palette selection in party config
- *(party)* different scheme for colors
- *(tui)* state management for palette selection UI
- *(tui)* standardize some key hints
- *(dev)* add dev command for unlocking all palettes for a party
- rename "color" -> "palette"

## [0.1.3](https://github.com/drewzemke/post-push-party/compare/v0.1.2...v0.1.3) - 2026-02-17

### Added

- *(cli)* change `status` subcommand to `points`
- *(cli)* add stats subcommand
- *(party)* simplify points breakdown if no commits counted
- *(party)* add color to points breakdown party
- *(party)* fireworks party stops after particles leave screen
- *(party)* add fireworks party

### Fixed

- *(party)* text alignment in stats party
- *(ci)* restore cache after installing jj
- *(party)* don't print extra blank line after fireworks party

### Other

- convert from workspace back to single package
- *(post-push-party)* release v0.1.3
- *(party)* extract utility functions for color/font control
- specificy readme in main package

## [0.1.2](https://github.com/drewzemke/post-push-party/compare/v0.1.1...v0.1.2) - 2026-02-15

### Other

- fix some lints
- add fireworks crate (unintegrated)
- convert to cargo workspace

## [0.1.1](https://github.com/drewzemke/post-push-party/compare/v0.1.0...v0.1.1) - 2026-02-14

### Added

- add initial readme

### Fixed

- *(tests)* use `jj git push` in test before installing party hook
- *(tests)* set default branch to main in test bare repos
- clippy lints
