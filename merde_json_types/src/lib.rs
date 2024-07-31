#![deny(missing_docs)]

//! `merde_json_types` is a companion crate to `merde_json`, providing wrapper
//! types, solving two problems at once:
//!
//!   - not all crates implement the `merde_json` traits, so a newtype
//!     is required anyway.
//!   - we might want to have some structs be part of our public interface,
//!     but only conditionally implement the `merde_json` traits (to avoid
//!     polluting the dependents' tree with `merde_json` if they don't need it).
//!
//! As a result, have your crate depend on `merde_json_types` unconditionally (which has
//! zero dependencies), and forward your own `merde_json` cargo feature to `merde_json_types/merde_json`, like so:
//!
//! ```toml
//! [dependencies]
//! merde_json_types = { version = "0.1", features = ["merde_json"] }
//!
//! [features]
//! merde_json = ["merde_json_types/merde_json"]
//! ```
//!
//! Then, in your crate, you can use the `merde_json_types` types, and they will
//! be conditionally implemented for you.
//!
//! For example, if you have a crate `my_crate` that depends on `merde_json_types`,
//! and you want to use the `time` crate's `OffsetDateTime` type, you can do:
//!
//! ```rust
//! use merde_json::{from_str, JsonSerialize, ToRustValue};
//! use merde_json_types::time::Rfc3339;
//!
//! let dt = Rfc3339(time::OffsetDateTime::now_utc());
//! let serialized = dt.to_json_string();
//! let deserialized: Rfc3339<time::OffsetDateTime> =
//!     merde_json::from_str(&serialized).unwrap().to_rust_value().unwrap();
//! assert_eq!(dt, deserialized);
//! ```

#[cfg(feature = "time-types")]
pub mod time;
