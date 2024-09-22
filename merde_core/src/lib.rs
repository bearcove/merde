mod cowstr;
pub use cowstr::CowStr;

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

mod deserialize2;
pub use deserialize2::ArrayStart;
pub use deserialize2::Deserialize;
pub use deserialize2::Deserializer;
pub use deserialize2::Event;
pub use deserialize2::EventType;

rubicon::compatibility_check! {
    ("merde_core_pkg_version", env!("CARGO_PKG_VERSION")),

    #[cfg(feature = "compact_str")]
    ("compact_str", "enabled")
}
