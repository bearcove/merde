# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
