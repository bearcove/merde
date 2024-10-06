mod cowstr;
pub use cowstr::CowStr;

mod cowbytes;
pub use cowbytes::CowBytes;

mod array;
pub use array::Array;

mod map;
pub use map::Map;

mod error;
pub use error::MerdeError;
pub use error::ValueType;

mod into_static;
pub use into_static::IntoStatic;

mod with_lifetime;
pub use with_lifetime::WithLifetime;

mod value;
pub use value::Value;

mod deserialize;
pub use deserialize::ArrayStart;
pub use deserialize::Deserialize;
pub use deserialize::DeserializeOwned;
pub use deserialize::Deserializer;
pub use deserialize::Event;
pub use deserialize::EventType;

rubicon::compatibility_check! {
    ("merde_core_pkg_version", env!("CARGO_PKG_VERSION")),

    #[cfg(feature = "compact_str")]
    ("compact_str", "enabled")

    #[cfg(feature = "compact_bytes")]
    ("compact_bytes", "enabled")
}
