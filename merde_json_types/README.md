[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde_json_types.svg)](https://crates.io/crates/merde_json_types)
[![docs.rs](https://docs.rs/merde_json_types/badge.svg)](https://docs.rs/merde_json_types)

![The merde_json logo: a glorious poop floating above a pair of hands](assets/merde-json-banner.png)

_Logo by [Misiasart](https://www.deviantart.com/misiasart)_

# merde_json_types

`merde_json_types` is a companion crate to [merde_json](https://crates.io/crates/merde_json),
providing wrapper types that solve two problems at once.

## Problem 1: Most crates have types that do not implement the `merde_json` traits

I'm thinking about the [time crate](https://crates.io/crates/time), the [chrono crate](https://crates.io/crates/chrono), [camino](https://crates.io/crates/camino), etc.

If you have, say, a `time::OffsetDateTime` in one of your structs,
then merde_json's derive macro will not work. You _are_ going to need
a wrapper of some sort, and that's the kind of type this crate provides.

If you enable the `time-serialize`, `time-deserialize`, and `merde_json`
features, you can do this:

```rust
use merde_json::{from_str, JsonSerialize, ToRustValue};
use merde_json_types::time::Rfc3339;

let dt = Rfc3339(time::OffsetDateTime::now_utc());
let serialized = dt.to_json_string();
let deserialized: Rfc3339<time::OffsetDateTime> =
    merde_json::from_str(&serialized).unwrap().to_rust_value().unwrap();
assert_eq!(dt, deserialized);
```

## Problem 2: Keeping `merde_json` optional

The [time::Rfc3339] type is exported by this crate as soon as the `time-types`
feature is enabled. But `merde_json_types` doesn't even depend on `merde_json`
(or provide serialization/deserialization implementations) unless you activate
its `merde_json` feature!

That means, you can have your crate unconditionally depend on `merde_json_types`,
and use `Rfc3339` in your public structs:

```rust
use merde_json::{Fantome, JsonSerialize, ToRustValue};
use merde_json_types::time::Rfc3339;

#[derive(Debug, PartialEq, Eq)]
pub struct Person<'src, 'val> {
    pub name: String,
    pub birth_date: Rfc3339<time::OffsetDateTime>,

    pub _boo: Fantome<'src, 'val>,
}

merde_json::derive! {
    impl (JsonSerialize, JsonDeserialize) for Person { name, birth_date }
}
```

And still only depend on `merde_json` when your _own_ feature gets activated:

```toml
[dependencies]
merde_json_types = "2"
merde_json = { version = "2", optional = true }

[features]
merde_json = ["dep:merde_json", "merde_json_types/merde_json"]
```

Of course, for that to work, we need to get rid of any unconditional mention of
`merde_json` in our code, which would become something like:

```rust
use std::marker::PhantomData;
use merde_json_types::time::Rfc3339;

#[derive(Debug, PartialEq, Eq)]
pub struct Person<'src, 'val> {
    pub name: String,
    pub birth_date: Rfc3339<time::OffsetDateTime>,

    /// This field still _has_ to be named `_boo`, but we can't use
    /// the `Fantome` type here without pulling in `merde_json`: so,
    /// we use `PhantomData` instead.
    pub _boo: PhantomData<(&'src (), &'val ())>,
}

#[cfg(feature = "merde_json")]
merde_json::derive! {
    impl (JsonSerialize, JsonDeserialize) for Person { name, birth_date }
}
```
