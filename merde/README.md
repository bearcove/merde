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

### Deriving impls, serializing, deserializing

`serde` has its own `Serialize` and `Deserialize` traits, which you can derive
with, well, derive macros:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    let serialized = serde_json::to_string(&point).unwrap();
    println!("serialized = {}", serialized);

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}
```

By contrast, `merde` provides declarative macros — impls for traits
like `ValueDeserialize`, `JsonSerialize` can be generated with `merde::derive!`:

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Point { x, y }
}

fn main() {
    let point = Point { x: 1, y: 2 };

    let serialized = merde_json::to_string(&point);
    println!("serialized = {}", serialized);

    let deserialized: Point = merde_json::from_str_via_value(&serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}
```

Downsides:

  * Only works for structs
  * You have to repeat the field list (but you'll get an error if you mess it up)
  * Feels a bit eerie

Upsides:

  * Fast compiles

`merde_json::from_str_via_value` goes through `merde::Value`, which isn't ideal for performance
— there's no compelling technical reason for it to stay that way, just gotta think about an
interface that works for various formats. A good thinking topic for future versions.

### Copy-on-write types
