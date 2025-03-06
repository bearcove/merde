use std::{
    borrow::Cow, collections::HashMap, future::Future, hash::BuildHasher, pin::Pin, sync::Arc,
};

use crate::{
    metastack::MetastackExt, Array, ArrayStart, CowBytes, CowStr, Event, Map, MapStart, MerdeError,
    Value,
};

pub trait Serializer {
    fn write<'fut>(
        &'fut mut self,
        ev: Event<'fut>,
    ) -> impl Future<Output = Result<(), MerdeError<'static>>> + 'fut;
}

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait DynSerializer {
    fn write<'fut>(
        &'fut mut self,
        ev: Event<'fut>,
    ) -> BoxFut<'fut, Result<(), MerdeError<'static>>>;
}

impl dyn DynSerializer {
    fn _assert_dyn_safe(_: Box<dyn DynSerializer>) {}
}

impl<S> DynSerializer for S
where
    S: Serializer,
{
    fn write<'fut>(
        &'fut mut self,
        ev: Event<'fut>,
    ) -> BoxFut<'fut, Result<(), MerdeError<'static>>> {
        Box::pin(Serializer::write(self, ev))
    }
}

pub trait DynSerializerExt {
    fn serialize<T: Serialize>(&mut self, t: &T) -> Result<(), MerdeError<'static>>;
    fn dyn_serialize(&mut self, t: &dyn DynSerialize) -> Result<(), MerdeError<'static>>;
}

impl<S> DynSerializerExt for S
where
    S: DynSerializer,
{
    fn serialize<T: Serialize>(&mut self, t: &T) -> Result<(), MerdeError<'static>> {
        T::serialize(t, self).run_sync_with_metastack()
    }

    fn dyn_serialize(&mut self, t: &dyn DynSerialize) -> Result<(), MerdeError<'static>> {
        DynSerialize::dyn_serialize(t, self).run_sync_with_metastack()
    }
}

pub trait Serialize {
    fn serialize<'fut>(
        &'fut self,
        serializer: &'fut mut dyn DynSerializer,
    ) -> impl Future<Output = Result<(), MerdeError<'static>>> + 'fut;
}

/// Dynamic dispatch version of [`Serialize`].
pub trait DynSerialize {
    /// Dynamic dispatch version of [`Serialize::serialize`].
    fn dyn_serialize<'fut>(
        &'fut self,
        serializer: &'fut mut dyn DynSerializer,
    ) -> BoxFut<'fut, Result<(), MerdeError<'static>>>;
}

impl<S> DynSerialize for S
where
    S: Serialize,
{
    fn dyn_serialize<'fut>(
        &'fut self,
        serializer: &'fut mut dyn DynSerializer,
    ) -> BoxFut<'fut, Result<(), MerdeError<'static>>> {
        Box::pin(Serialize::serialize(self, serializer))
    }
}

macro_rules! impl_trivial_serialize {
    ($ty:ty, $($rest:tt)*) => {
        impl_trivial_serialize!($ty);
        impl_trivial_serialize!($($rest)*);
    };

    ($ty:ty) => {
        impl Serialize for $ty {
            async fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn DynSerializer,
            ) -> Result<(), MerdeError<'static>> {
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
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        serializer.write(Event::Str(CowStr::Borrowed(self))).await
    }
}

impl<'s> Serialize for &'s str {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        serializer.write(Event::Str(CowStr::Borrowed(self))).await
    }
}

impl<'s> Serialize for CowStr<'s> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        serializer
            .write(Event::Str(CowStr::Borrowed(self.as_ref())))
            .await
    }
}

impl<'s> Serialize for Cow<'s, str> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        serializer
            .write(Event::Str(CowStr::Borrowed(self.as_ref())))
            .await
    }
}

impl<'s> Serialize for CowBytes<'s> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        serializer
            .write(Event::Bytes(CowBytes::Borrowed(self.as_ref())))
            .await
    }
}

impl<T: Serialize> Serialize for Option<T> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        match self {
            Some(value) => value.serialize(serializer).await,
            None => serializer.write(Event::Null).await,
        }
    }
}

impl<T: Serialize> Serialize for &[T] {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
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
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
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

impl<T: Serialize> Serialize for Box<T> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        (**self).serialize(serializer).await
    }
}

impl<T: Serialize> Serialize for Arc<T> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        (**self).serialize(serializer).await
    }
}

impl<K: Serialize, V: Serialize, BH: BuildHasher> Serialize for HashMap<K, V, BH> {
    async fn serialize<'fut>(
        &'fut self,
        serializer: &'fut mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
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

impl Serialize for Map<'_> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
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

impl Serialize for Array<'_> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
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

impl Serialize for Value<'_> {
    async fn serialize<'se>(
        &'se self,
        serializer: &'se mut dyn DynSerializer,
    ) -> Result<(), MerdeError<'static>> {
        match self {
            Value::I64(i) => serializer.write(Event::I64(*i)).await,
            Value::U64(u) => serializer.write(Event::U64(*u)).await,
            Value::Float(f) => serializer.write(Event::F64(f.into_inner())).await,
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

macro_rules! impl_serialize_for_tuple {
    ($($type_arg:ident),*) => {
        impl<$($type_arg: Serialize),*> Serialize for ($($type_arg),*,) {
            async fn serialize<'se>(
                &'se self,
                serializer: &'se mut dyn DynSerializer,
            ) -> Result<(), MerdeError<'static>> {
                serializer.write(Event::ArrayStart(ArrayStart {
                    size_hint: Some(count_tup!($($type_arg)*))
                })).await?;

                impl_serialize_for_tuple!(@inner self serializer _field => _field () ($($type_arg)*));
                serializer.write(Event::ArrayEnd).await
            }
        }
    };

    (@inner $self:ident $serializer:ident $fieldpat:pat => $field:ident ($($ignores:tt)*) ($x:ident $($xs:ident)*)) => {
        let ($($ignores)* $fieldpat, ..) = &$self;
        $field.serialize($serializer).await?;
        impl_serialize_for_tuple!(@inner $self $serializer $fieldpat => $field ($($ignores)* _,) ($($xs)*));
    };
    (@inner $self:ident $serializer:ident $fieldpat:pat => $field:ident ($($ignores:tt)*) ()) => {
        // we're done
    }
}

macro_rules! count_tup {
    () => { 0 };
    ($t:ident $($rest:ident)*) => { 1 + count_tup!($($rest)*) };
}

impl_serialize_for_tuple!(T0);
impl_serialize_for_tuple!(T0, T1);
impl_serialize_for_tuple!(T0, T1, T2);
impl_serialize_for_tuple!(T0, T1, T2, T3);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_serialize_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

#[cfg(test)]
mod tests;
