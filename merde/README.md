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

Picture this: a large JSON documents, with large strings, that don't use escape sequences.

Instead of allocating a separate `String` on the heap for each of these, `serde` lets you
borrow from the input string, either automatically when you use a type like `&str`:

```rust,ignore
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Name<'s> {
    first: &'s str,
    middle: &'s str,
    last: &'s str,
}

// etc.
```

Or explicitly when you use a copy-on-write type like `Cow<'s, str>`:

```rust,ignore
use serde::{Serialize, Deserialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
struct Name<'s> {
    #[serde(borrow)]
    first: Cow<'s, str>,
    #[serde(borrow)]
    middle: Cow<'s, str>,
    #[serde(borrow)]
    last: Cow<'s, str>,
}
```

`serde` is really flexible here, letting you have types with multiple lifetimes, not
all of which are related to the input string.

`merde` only handles the simplest of cases: structs without a lifetime parameter
are the simple case, since they're always owned / `'static` (by definition):

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug)]
struct Name {
    first: String,
    middle: String,
    last: String,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Name { first, middle, last }
}
```

But, as a treat, structs passed to `merde_derive!` can have exactly one lifetime
parameter, so that you may use string slices:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug)]
struct Name<'s> {
    first: &'s str,
    middle: &'s str,
    last: &'s str,
}

merde::derive! {
    //                                              👇
    impl (ValueDeserialize, JsonSerialize) for Name<'s> { first, middle, last }
    //                                              👆
}
```

Note that in the `merde_derive!` invocation, we _have_ to give it the lifetime parameter's
name — this ends up generating different code, that can borrow from the input.

Similarly, you can use the built-in `Cow<'s, str>` type, although merde provides and
recommends its own `CowStr<'s>` type:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug)]
struct Name<'s> {
    first: CowStr<'s>,
    middle: CowStr<'s>,
    last: CowStr<'s>,
}

merde::derive! {
    impl (ValueDeserialize, JsonSerialize) for Name<'s> { first, middle, last }
}
```

Which dereferences to `&str` like you'd expect, but instead of using `String` as its
owned type, it uses `compact_str::CompactStr`, which stores strings of up to 24
bytes inline!
