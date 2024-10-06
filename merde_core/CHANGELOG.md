# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [6.1.0](https://github.com/bearcove/merde/compare/merde_core-v6.0.2...merde_core-v6.1.0) - 2024-10-06

### Added

- Add support for HashMap<K, V, S> (for other S)

## [6.0.2](https://github.com/bearcove/merde/compare/merde_core-v6.0.1...merde_core-v6.0.2) - 2024-10-04

### Other

- Make MerdeJsonError implement IntoStatic + impl for Result

## [6.0.1](https://github.com/bearcove/merde/compare/merde_core-v6.0.0...merde_core-v6.0.1) - 2024-10-04

### Other

- Introduce from_str_owned in the json module
- Introduce DeserializeOwned trait

## [6.0.0](https://github.com/bearcove/merde/compare/merde_core-v5.1.0...merde_core-v6.0.0) - 2024-09-22

### Added

- [**breaking**] Include key name in error ([#73](https://github.com/bearcove/merde/pull/73))

### Other

- Add bytes type ([#76](https://github.com/bearcove/merde/pull/76))
- Remove ValueDeserialize macros
- Remove definition of ValueDeserialize
- Make option optional
- Convert example
- Move mixed example to deserialize
- Move away from ValueDeserialize
- Use UnexpectedEvent
- Deserializable => Deserialize, a-la serde
- Fix all tests
- Well that works
- okay hang on
- Play around with API
- mhmh
- poll failed you say
- add lifetimes to errors aw yiss
- des2 ideas

## [5.1.0](https://github.com/bearcove/merde/compare/merde_core-v5.0.5...merde_core-v5.1.0) - 2024-09-20

### Added

- Add JsonSerialize and ValueDeserialize impls for f32, f64

## [5.0.5](https://github.com/bearcove/merde/compare/merde_core-v5.0.4...merde_core-v5.0.5) - 2024-09-17

### Fixed

- require rubicon 3.4.9

## [5.0.4](https://github.com/bearcove/merde/compare/merde_core-v5.0.3...merde_core-v5.0.4) - 2024-09-17

### Fixed

- Require rubicon 3.4.8

## [5.0.3](https://github.com/bearcove/merde/compare/merde_core-v5.0.2...merde_core-v5.0.3) - 2024-09-17

### Other

- Run rubicon compatibility checks in various places around CowStr (deref, etc.)

## [5.0.2](https://github.com/bearcove/merde/compare/merde_core-v5.0.1...merde_core-v5.0.2) - 2024-09-17

### Other

- Add/fix logo attribution

## [5.0.1](https://github.com/bearcove/merde/compare/merde_core-v5.0.0...merde_core-v5.0.1) - 2024-09-16

### Other

- Add rusqlite ToSql/FromSql implementations for CowStr if the corresponding feature is enabled

## [5.0.0](https://github.com/bearcove/merde/compare/merde_core-v4.0.2...merde_core-v5.0.0) - 2024-09-15

### Added

- Introduce OwnedValueDeserialize
- [**breaking**] Introduce WithLifetime trait

### Other

- Implement ValueDeserialize for Box<T>, Rc<T>, Arc<T>
- Add rubicon compat check to merde_core, closes [#58](https://github.com/bearcove/merde/pull/58)
- Provide from_utf8 family of functions + AsRef<str> for CowStr

## [4.0.2](https://github.com/bearcove/merde/compare/merde_core-v4.0.1...merde_core-v4.0.2) - 2024-09-14

### Other

- Add more PartialEq implementations for CowStr to allow 'cow_str == blah'

## [4.0.1](https://github.com/bearcove/merde/compare/merde_core-v4.0.0...merde_core-v4.0.1) - 2024-09-14

### Other

- Add serde feature for merde/merde_core for CowStr

## [3.0.1](https://github.com/bearcove/merde/compare/merde_core-v3.0.0...merde_core-v3.0.1) - 2024-09-12

### Other

- Remove unused dependencies

## [2.2.3](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.2.2...merde_json_types-v2.2.3) - 2024-09-05

### Other
- Update logo attribution

## [2.2.2](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.2.1...merde_json_types-v2.2.2) - 2024-08-16

### Other
- updated the following local packages: merde_json, merde_json

## [2.2.1](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.2.0...merde_json_types-v2.2.1) - 2024-08-16

### Other
- updated the following local packages: merde_json, merde_json

## [2.2.0](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.1.2...merde_json_types-v2.2.0) - 2024-08-16

### Added
- Provide Fantome from both merde-json and merde-json-types

## [2.1.2](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.1.1...merde_json_types-v2.1.2) - 2024-08-16

### Other
- updated the following local packages: merde_json, merde_json

## [2.1.1](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.1.0...merde_json_types-v2.1.1) - 2024-07-31

### Other
- Use public URL, hopefully works on crates too?
- Add @2x asset
- Add logo

## [2.1.0](https://github.com/bearcove/merde_json/compare/merde_json_types-v2.0.0...merde_json_types-v2.1.0) - 2024-07-31

### Added
- Add From<T> for Rfc3339 wrapper

## [2.0.0](https://github.com/bearcove/merde_json/releases/tag/merde_json_types-v2.0.0) - 2024-07-31

### Added
- Introduce merde_json_types
