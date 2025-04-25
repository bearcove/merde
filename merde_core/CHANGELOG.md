# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [10.0.6](https://github.com/bearcove/merde/compare/merde_core-v10.0.5...merde_core-v10.0.6) - 2025-04-25

### Other

- Update ordered-float to v5.0.0

## [10.0.5](https://github.com/bearcove/merde/compare/merde_core-v10.0.4...merde_core-v10.0.5) - 2025-04-16

### Other

- mh

## [10.0.4](https://github.com/bearcove/merde/compare/merde_core-v10.0.3...merde_core-v10.0.4) - 2025-04-16

### Other

- IntoStatic for Utf8PathBuf

## [10.0.3](https://github.com/bearcove/merde/compare/merde_core-v10.0.2...merde_core-v10.0.3) - 2025-04-16

### Other

- Add support for camino

## [10.0.2](https://github.com/bearcove/merde/compare/merde_core-v10.0.1...merde_core-v10.0.2) - 2025-03-06

### Fixed

- make more calls to trait functions, or trait methods, fully qualified to avoid clenches with serde

## [10.0.1](https://github.com/bearcove/merde/compare/merde_core-v10.0.0...merde_core-v10.0.1) - 2025-01-29

### Other

- Add impls for Arc<T>

## [10.0.0](https://github.com/bearcove/merde/compare/merde_core-v9.0.1...merde_core-v10.0.0) - 2024-12-04

### Other

- [**breaking**] Force a major version bump
- remove rubicon (merde has a single ABI now anyway)

## [9.0.1](https://github.com/bearcove/merde/compare/merde_core-v9.0.0...merde_core-v9.0.1) - 2024-12-02

### Other

- Remove backtrace

## [9.0.0](https://github.com/bearcove/merde/compare/merde_core-v8.1.1...merde_core-v9.0.0) - 2024-11-30

### Other

- Introduce DynDeserialize
- Remove workaround for https://github.com/rust-lang/rust/issues/133676
- Mhh
- Use DynSerialize in merde_json
- Introduce DynSerialize
- Rename serialize_sync to serialize
- remove async versions of things
- wip dyn serialize
- More!
- yay other errors
- Dwip
- lift static requirement
- mhh lifetimes yes
- mh
- well that's hard
- Expose to_tokio_writer
- Remove JsonSerialize trait
- Expose an async Deserializer interface
- Make Deserializer::next async
- Move things around re: absorbing merde_time in merde_core
- Yus

## [8.1.1](https://github.com/bearcove/merde/compare/merde_core-v8.1.0...merde_core-v8.1.1) - 2024-11-24

### Other

- impl WIthLifetime for Option<T>

## [8.1.0](https://github.com/bearcove/merde/compare/merde_core-v8.0.0...merde_core-v8.1.0) - 2024-11-20

### Added

- Implement Deserialize and IntoStatic for `Box<T>` ([#107](https://github.com/bearcove/merde/pull/107))

## [8.0.0](https://github.com/bearcove/merde/compare/merde_core-v7.0.0...merde_core-v8.0.0) - 2024-11-04

### Added

- Impl WithLifetime for Value (woops)

### Other

- Make compact_str / compact_bytes non-optional
- Introduce Serialize trait
- As pointed out, FieldSlot must be invariant
- We did ask miri
- More tests around FieldSlot ([#101](https://github.com/bearcove/merde/pull/101))
- Don't allow trivial UB via FieldSlot in safe code
- I made miri unsad
- I made miri sad
- Add deserializer opinions, cf. [#89](https://github.com/bearcove/merde/pull/89)
- Introduce deserialization opinions
- macOS fixes
- Fix infinite stack linux support
- Oh yeah our MSRV is 1.75 because AFIT
- fine let's not make msrv rust 1.82
- Actually query the stack size, don't hardcode anything
- Comments--
- The trick actually works
- Committing before something bad happens
- Start the trick
- Deserialize borrowed variants of cowstr

## [7.0.0](https://github.com/bearcove/merde/compare/merde_core-v6.1.0...merde_core-v7.0.0) - 2024-10-06

### Added

- Implement Eq for values

### Other

- Fix tests
- Add support for msgpack deserialization

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
