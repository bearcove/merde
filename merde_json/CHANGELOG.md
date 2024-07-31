# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
