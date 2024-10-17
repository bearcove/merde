mod cowstr;
use std::cell::Cell;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;

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

type BoxFuture = Pin<Box<dyn Future<Output = ()>>>;

std::thread_local! {
    pub static STACK_BASE: Cell<u64> = const { Cell::new(0) };
    pub static NEXT_FUTURE: RefCell<Option<BoxFuture>> = const { RefCell::new(None) };
}

rubicon::compatibility_check! {
    ("merde_core_pkg_version", env!("CARGO_PKG_VERSION")),

    #[cfg(feature = "compact_str")]
    ("compact_str", "enabled")

    #[cfg(feature = "compact_bytes")]
    ("compact_bytes", "enabled")
}
