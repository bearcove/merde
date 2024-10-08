# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [6.2.1](https://github.com/bearcove/merde/compare/merde-v6.2.0...merde-v6.2.1) - 2024-10-07

### Fixed

- Proper starter handling in merde_msgpack

## [6.2.0](https://github.com/bearcove/merde/compare/merde-v6.1.0...merde-v6.2.0) - 2024-10-06

### Added

- Implement Eq for values

### Other

- Fix tests
- Add support for msgpack deserialization

## [6.1.0](https://github.com/bearcove/merde/compare/merde-v6.0.4...merde-v6.1.0) - 2024-10-06

### Added

- Add support for HashMap<K, V, S> (for other S)

## [6.0.4](https://github.com/bearcove/merde/compare/merde-v6.0.3...merde-v6.0.4) - 2024-10-04

### Other

- Fix empty objects / empty arrays

## [6.0.3](https://github.com/bearcove/merde/compare/merde-v6.0.2...merde-v6.0.3) - 2024-10-04

### Other

- updated the following local packages: merde_core, merde_json

## [6.0.2](https://github.com/bearcove/merde/compare/merde-v6.0.1...merde-v6.0.2) - 2024-10-04

### Other

- Introduce DeserializeOwned trait

## [6.0.1](https://github.com/bearcove/merde/compare/merde-v6.0.0...merde-v6.0.1) - 2024-10-01

### Other

- respect StreamEnd
- merde_yaml is v6

## [6.0.0](https://github.com/bearcove/merde/compare/merde-v5.1.1...merde-v6.0.0) - 2024-09-22

### Added

- [**breaking**] Include key name in error ([#73](https://github.com/bearcove/merde/pull/73))

### Other

- Initial merde_yaml addition ([#77](https://github.com/bearcove/merde/pull/77))
- Remove ValueDeserialize macros
- Make option optional
- Port more things to deserialize
- Steal @compiler-errors's suggestion (thanks Michael!)
- port one more example
- impl_deserialize is a noop unless the feature is enabled
- Convert example
- Move mixed example to deserialize
- Move more examples over to Deserialize
- Move away from ValueDeserialize
- Fix all tests
- add lifetimes to errors aw yiss

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
