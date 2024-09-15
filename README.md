[![license: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![crates.io](https://img.shields.io/crates/v/merde_json.svg)](https://crates.io/crates/merde_json)
[![docs.rs](https://docs.rs/merde_json/badge.svg)](https://docs.rs/merde_json)

# merde

![The merde logo: a glorious poop floating above a pair of hands](https://github.com/user-attachments/assets/763d60e0-5101-48af-bc72-f96f516a5d0f)

_Logo by [MisiasArt](https://misiasart.carrd.co)_

A simpler (and slightly shittier) take on [serde](https://crates.io/crates/serde)

Do you want to deal with JSON data? Are you not _that_ worried about the
performance overhead? (ie. you're writing a backend in Rust, but if it was
written in Node.js nobody would bat an eye?).

Do you value short build times at the expense of some comfort?

Then head over to the crate documentations:

  * [merde](./merde/README.md)
  * [merde_json](./merde_json/README.md)

## FAQ

### What's with the name?

It's pronounced "murr-day", because we're merializing and demerializing things.

It's also something you may hear a French person yell when they're sick of waiting
for things to build, just before "j'en ai marre!!"

### Why?

I value iteration times (re builds, etc.) more than I value microseconds saved deserializing
JSON payloads — I got tired of proc macros getting compiled, parsing all my code, generating
a ton of generic code of their own, etc.

I also wanted a nice, ergonomic `Value` type that isn't _quite_ tied to JSON, for when you
just can't be arsed deriving your own structs.

The declarative macro approach is less flexible and not so DRY but so much lighter. Some more
subtlety might be added later, who knows.

## License

This project is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
