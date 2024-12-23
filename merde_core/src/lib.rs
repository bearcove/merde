mod cowstr;

pub use cowstr::CowStr;

mod covariance_proofs;

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

mod metastack;
pub use metastack::{with_metastack_resume_point, MetastackExt};

mod event;
pub use event::ArrayStart;
pub use event::Event;
pub use event::EventType;
pub use event::MapStart;

mod serialize;
pub use serialize::DynSerialize;
pub use serialize::DynSerializer;
pub use serialize::DynSerializerExt;
pub use serialize::Serialize;
pub use serialize::Serializer;

mod deserialize;
pub use deserialize::DefaultDeserOpinions;
pub use deserialize::DeserOpinions;
pub use deserialize::Deserialize;
pub use deserialize::DeserializeOwned;
pub use deserialize::Deserializer;
pub use deserialize::DynDeserialize;
pub use deserialize::DynDeserializer;
pub use deserialize::DynDeserializerExt;
pub use deserialize::FieldSlot;

pub mod time;
