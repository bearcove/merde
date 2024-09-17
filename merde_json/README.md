[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde_json.svg)](https://crates.io/crates/merde_json)
[![docs.rs](https://docs.rs/merde_json/badge.svg)](https://docs.rs/merde_json)

# merde_json

![The merde logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

_Logo by [MisiasArt](https://misiasart.com)_

Adds JSON serialization/deserialization support for
[merde](https://crates.io/crates/merde).

You would normally add a dependency on [merde](https://crates.io/crates/merde)
directly, enabling its `json` feature.

## Implementation

The underlying parser (including aarch64 SIMD support, bignum support, etc.) has been
taken wholesale from the [jiter crate](https://crates.io/crates/jiter) for now.

[An issue has been opened](https://github.com/pydantic/jiter/issues/139) to discuss sharing a core.
