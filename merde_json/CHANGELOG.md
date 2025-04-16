# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [10.0.5](https://github.com/bearcove/merde/compare/merde_json-v10.0.4...merde_json-v10.0.5) - 2025-04-16

### Other

- updated the following local packages: merde_core

## [10.0.4](https://github.com/bearcove/merde/compare/merde_json-v10.0.3...merde_json-v10.0.4) - 2025-04-16

### Other

- updated the following local packages: merde_core

## [10.0.3](https://github.com/bearcove/merde/compare/merde_json-v10.0.2...merde_json-v10.0.3) - 2025-04-16

### Other

- updated the following local packages: merde_core

## [10.0.2](https://github.com/bearcove/merde/compare/merde_json-v10.0.1...merde_json-v10.0.2) - 2025-03-06

### Other

- updated the following local packages: merde_core

## [10.0.1](https://github.com/bearcove/merde/compare/merde_json-v10.0.0...merde_json-v10.0.1) - 2025-01-29

### Other

- updated the following local packages: merde_core

## [10.0.0](https://github.com/bearcove/merde/compare/merde_json-v9.0.1...merde_json-v10.0.0) - 2024-12-04

### Other

- [**breaking**] Force a major version bump

## [9.0.1](https://github.com/bearcove/merde/compare/merde_json-v9.0.0...merde_json-v9.0.1) - 2024-12-02

### Other

- updated the following local packages: merde_core

## [9.0.0](https://github.com/bearcove/merde/compare/merde_json-v8.0.2...merde_json-v9.0.0) - 2024-11-30

### Other

- Mhh
- Use DynSerialize in merde_json
- Rename serialize_sync to serialize
- async stuff Does Not Work for now
- remove async versions of things
- wip dyn serialize
- bye bigint
- Remove unused jiter_lite methods
- Fix more warnings and errors
- More!
- yay other errors
- Dwip
- Expose to_tokio_writer
- Remove JsonSerialize trait
- Expose an async Deserializer interface
- Make Deserializer::next async
- Move things around re: absorbing merde_time in merde_core

## [8.0.2](https://github.com/bearcove/merde/compare/merde_json-v8.0.1...merde_json-v8.0.2) - 2024-11-24

### Other

- updated the following local packages: merde_core

## [8.0.1](https://github.com/bearcove/merde/compare/merde_json-v8.0.0...merde_json-v8.0.1) - 2024-11-20

### Other

- updated the following local packages: merde_core

## [8.0.0](https://github.com/bearcove/merde/compare/merde_json-v6.2.1...merde_json-v8.0.0) - 2024-11-04

### Other

- Introduce Serialize trait

## [6.2.1](https://github.com/bearcove/merde/compare/merde_json-v6.2.0...merde_json-v6.2.1) - 2024-10-07

### Fixed

- Proper starter handling in merde_msgpack

## [6.2.0](https://github.com/bearcove/merde/compare/merde_json-v6.1.0...merde_json-v6.2.0) - 2024-10-06

### Added

- Implement Eq for values

### Other

- Fix tests
- Add support for msgpack deserialization

## [6.1.0](https://github.com/bearcove/merde/compare/merde_json-v6.0.3...merde_json-v6.1.0) - 2024-10-06

### Added

- Add support for HashMap<K, V, S> (for other S)
- Remove debug prints, provide yaml::from_str/owned

## [6.0.3](https://github.com/bearcove/merde/compare/merde_json-v6.0.2...merde_json-v6.0.3) - 2024-10-04

### Other

- Fix empty objects / empty arrays

## [6.0.2](https://github.com/bearcove/merde/compare/merde_json-v6.0.1...merde_json-v6.0.2) - 2024-10-04

### Other

- Make MerdeJsonError implement IntoStatic + impl for Result

## [6.0.1](https://github.com/bearcove/merde/compare/merde_json-v6.0.0...merde_json-v6.0.1) - 2024-10-04

### Other

- Introduce from_str_owned in the json module

## [6.0.0](https://github.com/bearcove/merde/compare/merde_json-v5.1.0...merde_json-v6.0.0) - 2024-09-22

### Other

- Add bytes type ([#76](https://github.com/bearcove/merde/pull/76))
- Remove ValueDeserialize macros
- Remove definition of ValueDeserialize
- Convert example
- Move away from ValueDeserialize
- Use UnexpectedEvent
- Deserializable => Deserialize, a-la serde
- Fix all tests
- Well that works
- okay hang on
- get rid of queue in JsonSerializer
- Play around with API
- mhmh
- Well the new deserializer seems to be working
- poll failed you say
- add lifetimes to errors aw yiss

## [5.1.0](https://github.com/bearcove/merde/compare/merde_json-v5.0.5...merde_json-v5.1.0) - 2024-09-20

### Added

- Add JsonSerialize and ValueDeserialize impls for f32, f64

## [5.0.5](https://github.com/bearcove/merde/compare/merde_json-v5.0.4...merde_json-v5.0.5) - 2024-09-17

### Other

- updated the following local packages: merde_core

## [5.0.4](https://github.com/bearcove/merde/compare/merde_json-v5.0.3...merde_json-v5.0.4) - 2024-09-17

### Other

- updated the following local packages: merde_core

## [5.0.3](https://github.com/bearcove/merde/compare/merde_json-v5.0.2...merde_json-v5.0.3) - 2024-09-17

### Other

- updated the following local packages: merde_core

## [5.0.2](https://github.com/bearcove/merde/compare/merde_json-v5.0.1...merde_json-v5.0.2) - 2024-09-17

### Other

- Add/fix logo attribution

## [5.0.1](https://github.com/bearcove/merde/compare/merde_json-v5.0.0...merde_json-v5.0.1) - 2024-09-16

### Other

- updated the following local packages: merde_core

## [5.0.0](https://github.com/bearcove/merde/compare/merde_json-v4.0.2...merde_json-v5.0.0) - 2024-09-15

### Added

- Introduce OwnedValueDeserialize
- [**breaking**] Introduce WithLifetime trait

## [4.0.2](https://github.com/bearcove/merde/compare/merde_json-v4.0.1...merde_json-v4.0.2) - 2024-09-14

### Other

- updated the following local packages: merde_core

## [4.0.1](https://github.com/bearcove/merde/compare/merde_json-v4.0.0...merde_json-v4.0.1) - 2024-09-14

### Other

- updated the following local packages: merde_core

## [3.0.1](https://github.com/bearcove/merde/compare/merde_json-v3.0.0...merde_json-v3.0.1) - 2024-09-12

### Other

- updated the following local packages: merde_core

## [2.4.1](https://github.com/bearcove/merde_json/compare/merde_json-v2.4.0...merde_json-v2.4.1) - 2024-09-05

### Other
- Update logo attribution

## [2.4.0](https://github.com/bearcove/merde_json/compare/merde_json-v2.3.1...merde_json-v2.4.0) - 2024-08-16

### Added
- Implement ToStatic for String

## [2.3.1](https://github.com/bearcove/merde_json/compare/merde_json-v2.3.0...merde_json-v2.3.1) - 2024-08-16

### Fixed
- Remove (dev) dep on serde_json

## [2.3.0](https://github.com/bearcove/merde_json/compare/merde_json-v2.2.0...merde_json-v2.3.0) - 2024-08-16

### Added
- Provide Fantome from both merde-json and merde-json-types

## [2.2.0](https://github.com/bearcove/merde_json/compare/merde_json-v2.1.2...merde_json-v2.2.0) - 2024-08-16

### Added
- Impl ToStatic for more standard collection types

### Other
- Run examples in CI

## [2.1.2](https://github.com/bearcove/merde_json/compare/merde_json-v2.1.1...merde_json-v2.1.2) - 2024-07-31

### Other
- Use public URL, hopefully works on crates too?
- Add @2x asset
- Add logo

## [2.1.1](https://github.com/bearcove/merde_json/compare/merde_json-v2.1.0...merde_json-v2.1.1) - 2024-07-31

### Other
- Move CHANGELOG in the right place

## [2.0.0](https://github.com/bearcove/merde_json/compare/v1.0.1...v2.0.0) - 2024-07-31

### Added
- Introduce `to_string` and other top-level functions for serde_json compat
- Implement ToStatic for Option<T>

### Other
- I guess that bound wasn't necessary
- Elide lifetimes
- Tests pass! Let's only do OffsetDateTime
- Some unit tests for datetime (failing so far)
- Make both enums non-exhaustive
- WIP time implementation
- Also run on merge_group

## [1.0.1](https://github.com/bearcove/merde_json/compare/v1.0.0...v1.0.1) - 2024-07-29

### Fixed
- Declare lifetime parameters in a consistent order, always ([#4](https://github.com/bearcove/merde_json/pull/4))

### Other
- release

## [1.0.0](https://github.com/bearcove/merde_json/releases/tag/v1.0.0) - 2024-07-29

### Other
- Add release-plz flow
- Alright then
- Flesh out README, add funding
- Don't need the rust action?
- All tests pass I believe
- Mhmh
- More tests pass
- Show off ToStatic
- mh
- Add mixed example
- Getting somewhere
- Fix CI workflow
- Lift 'inner at the trait level for JsonDeserialize
- More docs
- Initial import
