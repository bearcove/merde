use std::{borrow::Cow, collections::HashMap, future::Future, hash::BuildHasher};

use crate::{
    metastack::MetastackExt, Array, ArrayStart, CowBytes, CowStr, Event, Map, MapStart, Value,
};

pub trait Serializer {
    type Error;

    // (note: this is an async fn but because there's a lifetime, it won't let us!)
    fn write(&mut self, ev: Event<'_>) -> impl Future<Output = Result<(), Self::Error>>;

    fn serialize<T: Serialize>(&mut self, t: &T) -> Result<(), Self::Error> {
        Serialize::serialize(t, self).run_synchronously_with_metastack()
    }
}

pub trait Serialize {
    #[allow(async_fn_in_trait)]
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized;
}

macro_rules! impl_trivial_serialize {
    ($ty:ty, $($rest:tt)*) => {
        impl_trivial_serialize!($ty);
        impl_trivial_serialize!($($rest)*);
    };

    ($ty:ty) => {
        impl Serialize for $ty {
            async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where
                S: Serializer + ?Sized,
            {
                serializer.write(Event::from(*self)).await
            }
        }
    };

    (,) => {};
    () => {};
}

impl_trivial_serialize! {
    i8, i16, i32, i64,
    u8, u16, u32, u64,
    isize, usize,
    // floats
    f32, f64,
    // misc.
    bool,
}

impl Serialize for String {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer.write(Event::Str(CowStr::Borrowed(self))).await
    }
}

impl<'s> Serialize for &'s str {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer.write(Event::Str(CowStr::Borrowed(self))).await
    }
}

impl<'s> Serialize for CowStr<'s> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::Str(CowStr::Borrowed(self.as_ref())))
            .await
    }
}

impl<'s> Serialize for Cow<'s, str> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::Str(CowStr::Borrowed(self.as_ref())))
            .await
    }
}

impl<'s> Serialize for CowBytes<'s> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::Bytes(CowBytes::Borrowed(self.as_ref())))
            .await
    }
}

impl<T: Serialize> Serialize for Option<T> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        match self {
            Some(value) => value.serialize(serializer).await,
            // well that's uhhh questionable — in JS,
            // null != undefined. other formats might not have
            // a concept of null, or deal with optional fields
            // completely differently.
            // I guess we're assuming that if you're calling
            // `.serialize()` on a `None` option, you're comfortable
            // receiving a `Null`? this needs to be documented, there's
            // some design work here to make sure this works for
            // non-self-descriptive formats
            None => serializer.write(Event::Null).await,
        }
    }
}

impl<T: Serialize> Serialize for &[T] {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::ArrayStart(ArrayStart {
                size_hint: Some(self.len()),
            }))
            .await?;
        for item in *self {
            item.serialize(serializer).await?;
        }
        serializer.write(Event::ArrayEnd).await
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::ArrayStart(ArrayStart {
                size_hint: Some(self.len()),
            }))
            .await?;
        for item in self {
            item.serialize(serializer).await?;
        }
        serializer.write(Event::ArrayEnd).await
    }
}

impl<K: Serialize, V: Serialize, BH: BuildHasher> Serialize for HashMap<K, V, BH> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::MapStart(MapStart {
                size_hint: Some(self.len()),
            }))
            .await?;
        for (key, value) in self {
            key.serialize(serializer).await?;
            value.serialize(serializer).await?;
        }
        serializer.write(Event::MapEnd).await
    }
}

impl<'s> Serialize for Map<'s> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::MapStart(MapStart {
                size_hint: Some(self.len()),
            }))
            .await?;
        for (key, value) in self.iter() {
            serializer.write(Event::Str(CowStr::Borrowed(key))).await?;
            value.serialize(serializer).await?;
        }
        serializer.write(Event::MapEnd).await
    }
}

impl<'s> Serialize for Array<'s> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        serializer
            .write(Event::ArrayStart(ArrayStart {
                size_hint: Some(self.len()),
            }))
            .await?;
        for item in self.iter() {
            item.serialize(serializer).await?;
        }
        serializer.write(Event::ArrayEnd).await
    }
}

impl<'s> Serialize for Value<'s> {
    async fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer + ?Sized,
    {
        match self {
            Value::I64(i) => serializer.write(Event::I64(*i)).await,
            Value::U64(u) => serializer.write(Event::U64(*u)).await,
            Value::Float(f) => serializer.write(Event::Float(f.into_inner())).await,
            Value::Str(s) => serializer.write(Event::Str(s.clone())).await,
            Value::Bytes(b) => serializer.write(Event::Bytes(b.clone())).await,
            Value::Null => serializer.write(Event::Null).await,
            Value::Bool(b) => serializer.write(Event::Bool(*b)).await,
            Value::Array(arr) => {
                arr.serialize(serializer)
                    .with_metastack_resume_point()
                    .await
            }
            Value::Map(map) => {
                map.serialize(serializer)
                    .with_metastack_resume_point()
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests;
