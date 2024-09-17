[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde_core.svg)](https://crates.io/crates/merde_core)
[![docs.rs](https://docs.rs/merde_core/badge.svg)](https://docs.rs/merde_core)

# merde_core

![The merde logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

_Logo by [MisiasArt](https://misiasart.com)_

The `merde` family of crates aims to provide a lighter, simpler, and
build-time-friendly alternative to `serde`.

This "core" crate provides core types like `Value`, `Array`, `Map`,
and `CowStr<'s>` (a copy-on-write string type that also leverages
[compact_str](https://crates.io/crates/compact_str)'s small string
optimization), and traits like `ValueDeserialize` and `IntoStatic`.

Crates that provide support for formats (like [merde_json](https://crates.io/crates/merde_json)),
and crates that provide wrappers around other crates' types, to allow serializing/deserializing
them (like [merde_time](https://crates.io/crates/merde_time)), depend only on the "core" crate.

The umbrella crate [merde](https://crates.io/crates/merde) re-exports core's types, along
with a `derive!` macro which lets you implement `ValueDeserialize`, `IntoStatic`, and format-specific
traits like `JsonSerialize` on structs, with or without lifetime parameters.
