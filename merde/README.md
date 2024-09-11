[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde.svg)](https://crates.io/crates/merde)
[![docs.rs](https://docs.rs/merde/badge.svg)](https://docs.rs/merde)

# merde

[The merde logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

`merde` aims to provide a simpler, lighter alternative to [serde](https://crates.io/crates/serde),
that might run slower, but compiles faster.

This is the "hub" crate, which re-exports all types from [merde_core](https://crates.io/crates/merde_core),
including `Value`, `Array`, and `Object`, and provides a declarative `derive!` macro that helps implement
traits like `ValueDeserialize`, `ToStatic` and `JsonSerialize`.

## From `serde` to `merde`

`serde` has its own `Serialize` and `Deserialize` traits, which you can derive
with, well, derive macros:

```rust no_compile
```
