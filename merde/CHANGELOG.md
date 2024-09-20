# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.1.1](https://github.com/bearcove/merde/compare/merde-v5.1.0...merde-v5.1.1) - 2024-09-20

### Other

- updated the following local packages: merde_core, merde_json

## [5.1.0](https://github.com/bearcove/merde/compare/merde-v5.0.5...merde-v5.1.0) - 2024-09-20

### Added

- Add support for string-like enums

## [5.0.5](https://github.com/bearcove/merde/compare/merde-v5.0.4...merde-v5.0.5) - 2024-09-17

### Other

- update Cargo.lock dependencies

## [5.0.4](https://github.com/bearcove/merde/compare/merde-v5.0.3...merde-v5.0.4) - 2024-09-17

### Other

- update Cargo.lock dependencies

## [5.0.3](https://github.com/bearcove/merde/compare/merde-v5.0.2...merde-v5.0.3) - 2024-09-17

### Other

- update Cargo.lock dependencies

## [5.0.2](https://github.com/bearcove/merde/compare/merde-v5.0.1...merde-v5.0.2) - 2024-09-17

### Other

- updated the following local packages: merde_core, merde_json, merde_time

## [5.0.1](https://github.com/bearcove/merde/compare/merde-v5.0.0...merde-v5.0.1) - 2024-09-16

### Other

- Don't depend on merde_time by default
- Add rusqlite ToSql/FromSql implementations for CowStr if the corresponding feature is enabled

## [5.0.0](https://github.com/bearcove/merde/compare/merde-v4.0.5...merde-v5.0.0) - 2024-09-15

### Added

- Introduce OwnedValueDeserialize
- [**breaking**] Introduce WithLifetime trait

### Other

- Doc for externally-tagged enums
- Add doc in derive for tuple structs
- Add doc for enums & tuple structs
- done with tuple structs
- rejiggle order
- wip tuple structs
- Allow deriving for externally-tagged enums
- WIP enum support
- Require 'struct' prefix when deriving valuedeserialize etc.
- Introduce WithLifetime trait (thanks @JaSpa)
- Showcase 'impl is not general enough' problem

## [4.0.5](https://github.com/bearcove/merde/compare/merde-v4.0.4...merde-v4.0.5) - 2024-09-14

### Other

- one more cfg-gate lacking

## [4.0.4](https://github.com/bearcove/merde/compare/merde-v4.0.3...merde-v4.0.4) - 2024-09-14

### Other

- Make merde_time flags make sense

## [4.0.3](https://github.com/bearcove/merde/compare/merde-v4.0.2...merde-v4.0.3) - 2024-09-14

### Other

- Pull feature gates outside macros

## [4.0.2](https://github.com/bearcove/merde/compare/merde-v4.0.1...merde-v4.0.2) - 2024-09-14

### Other

- updated the following local packages: merde_core

## [4.0.1](https://github.com/bearcove/merde/compare/merde-v4.0.0...merde-v4.0.1) - 2024-09-14

### Other

- Add serde feature for merde/merde_core for CowStr

## [3.1.1](https://github.com/bearcove/merde/compare/merde-v3.1.0...merde-v3.1.1) - 2024-09-12

### Other

- Fix logo
