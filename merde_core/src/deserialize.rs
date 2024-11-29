use std::{
    any::TypeId,
    borrow::Cow,
    collections::HashMap,
    future::Future,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    pin::Pin,
};

use crate::{Array, CowStr, Event, EventType, IntoStatic, Map, MerdeError, Value};

pub struct DeserializerStartingWith<'borrow, 's> {
    pub starter: Option<Event<'s>>,
    pub next: &'borrow mut dyn DynDeserializer<'s>,
}

pub trait StartingWith<'s> {
    fn starting_with(&mut self, starter: Event<'s>) -> DeserializerStartingWith<'_, 's>;
}

impl<'s> StartingWith<'s> for dyn DynDeserializer<'s> {
    fn starting_with(&mut self, starter: Event<'s>) -> DeserializerStartingWith<'_, 's> {
        DeserializerStartingWith {
            starter: Some(starter),
            next: self,
        }
    }
}

impl std::fmt::Debug for DeserializerStartingWith<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeserializerStartingWith")
            .field("starter", &self.starter)
            .finish()
    }
}

impl<'s> Deserializer<'s> for DeserializerStartingWith<'_, 's> {
    async fn next(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        if let Some(ev) = self.starter.take() {
            return Ok(ev);
        }

        self.next.next().await
    }
}

pub trait Deserializer<'s>: std::fmt::Debug {
    /// Get the next event from the deserializer.
    #[doc(hidden)]
    fn next(&mut self) -> impl Future<Output = Result<Event<'s>, MerdeError<'s>>> + '_;
}

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait DynDeserializer<'s> {
    fn next(&mut self) -> BoxFut<'_, Result<Event<'s>, MerdeError<'s>>>;
}

impl dyn DynDeserializer<'_> {
    fn _assert_dyn_safe(_: Box<dyn DynDeserializer<'_>>) {}
}

impl<'s, D> DynDeserializer<'s> for D
where
    D: Deserializer<'s>,
{
    fn next(&mut self) -> BoxFut<'_, Result<Event<'s>, MerdeError<'s>>> {
        Box::pin(Deserializer::next(self))
    }
}

mod mini_typeid {
    // vendored straight from https://github.com/dtolnay/typeid — which is dual-licensed under
    // MIT and Apache-2.0, just like merde.
    //
    // We don't really need const type_id construction or older rustc support, so this is a minimal
    // take on it.

    use std::{any::TypeId, marker::PhantomData};

    #[must_use]
    #[inline(always)]
    pub fn of<T>() -> TypeId
    where
        T: ?Sized,
    {
        trait NonStaticAny {
            fn get_type_id(&self) -> TypeId
            where
                Self: 'static;
        }

        impl<T: ?Sized> NonStaticAny for PhantomData<T> {
            #[inline(always)]
            fn get_type_id(&self) -> TypeId
            where
                Self: 'static,
            {
                TypeId::of::<T>()
            }
        }

        let phantom_data = PhantomData::<T>;
        NonStaticAny::get_type_id(unsafe {
            std::mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom_data)
        })
    }
}

/// Allows filling in a field of a struct while deserializing.
pub struct FieldSlot<'s, 'borrow: 's> {
    option: *mut Option<()>,
    type_id_of_field: TypeId,
    type_name_of_field: &'static str,
    _phantom: PhantomData<&'borrow mut &'s mut ()>,
}

impl<'s, 'borrow: 's> FieldSlot<'s, 'borrow> {
    /// Construct a new `FieldSlot`, ready to be filled
    #[inline(always)]
    #[doc(hidden)]
    pub fn new<T: 's>(option: &'borrow mut Option<T>) -> Self {
        Self {
            option: unsafe {
                std::mem::transmute::<*mut Option<T>, *mut Option<()>>(option as *mut _)
            },
            type_id_of_field: mini_typeid::of::<T>(),
            type_name_of_field: std::any::type_name::<T>(),
            _phantom: PhantomData,
        }
    }

    /// Fill this field with a value.
    pub fn fill<T: 's>(self, value: T) {
        let type_id_of_value = mini_typeid::of::<T>();
        assert_eq!(
            self.type_id_of_field,
            type_id_of_value,
            "tried to assign a \x1b[33m{}\x1b[0m to a slot of type \x1b[34m{}\x1b[0m",
            std::any::type_name::<T>(),
            self.type_name_of_field,
        );

        unsafe {
            let option_ptr: *mut Option<T> = std::mem::transmute(self.option);
            (*option_ptr).replace(value);
        }
    }
}

/// Opinions you have about deserialization: should unknown fields
/// be allowed, etc.
///
/// These are opinions _for a specific type_, not for the whole
/// deserialization tree. They cannot be set from the outside, they
/// can only be used to control the behavior of code generated via
/// `merde::derive!`.
pub trait DeserOpinions {
    /// Should `{ a: 1, b: 2 }` be rejected when encountering b,
    /// if we are deserializing `struct Foo { a: i32 }`?
    fn deny_unknown_fields(&self) -> bool;

    /// If we encounter `{ "jazzBand": 1 }`, should we try to find a field
    /// named "jazzBand" on the struct we're deserializing, or should we
    /// map it to something else, like "jazz_band"?
    fn map_key_name<'s>(&self, key: CowStr<'s>) -> CowStr<'s>;

    /// If we encounter `{ a: 1 }`, but we are deserializing `struct Foo { a: i32, b: i32 }`,
    /// `fill_default` will be called with `key = "b"` and decide what to do.
    ///
    /// Note that this is called with the field name, not whatever name we found in the
    /// "document" — if `map_key_name` mapped "jazzBand" to "jazz_band", then this is
    /// called with "jazz_band".
    #[allow(clippy::needless_lifetimes)]
    fn default_field_value<'s, 'borrow>(&self, key: &'borrow str, slot: FieldSlot<'s, 'borrow>);
}

/// merde's default opinions for deserialization: allow unknown fields, don't fill in default values
/// and keep key names as-is.
pub struct DefaultDeserOpinions;

impl DeserOpinions for DefaultDeserOpinions {
    #[inline(always)]
    fn deny_unknown_fields(&self) -> bool {
        // by default, allow unknown fields
        false
    }

    #[inline(always)]
    #[allow(clippy::needless_lifetimes)]
    fn default_field_value<'s, 'borrow>(&self, _key: &'borrow str, _slot: FieldSlot<'s, 'borrow>) {
        // by default, don't fill in default values for any fields
        // (they will just error out)
    }

    #[inline(always)]
    fn map_key_name<'s>(&self, key: CowStr<'s>) -> CowStr<'s> {
        // by default, keep key names as-is
        key
    }
}

pub trait Deserialize<'s>: Sized {
    fn deserialize<'d>(
        de: &'d mut dyn DynDeserializer<'s>,
    ) -> impl Future<Output = Result<Self, MerdeError<'s>>> + 'd;

    fn from_option(value: Option<Self>, field_name: CowStr<'s>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(value) => Ok(value),
            None => Err(MerdeError::MissingProperty(field_name)),
        }
    }
}

pub trait DeserializeOwned<'s>: Sized + IntoStatic {
    fn deserialize_owned(
        de: &mut dyn DynDeserializer<'s>,
    ) -> impl Future<Output = Result<<Self as IntoStatic>::Output, MerdeError<'s>>>;
}

impl<'s, T> DeserializeOwned<'s> for T
where
    T: IntoStatic + Deserialize<'s>,
{
    async fn deserialize_owned(
        de: &mut dyn DynDeserializer<'s>,
    ) -> Result<<Self as IntoStatic>::Output, MerdeError<'s>> {
        T::deserialize(de).await.map(|v| v.into_static())
    }
}

impl<'s> Deserialize<'s> for i64 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: i64 = match de.next().await? {
            Event::I64(i) => i,
            Event::U64(u) => u.try_into().map_err(|_| MerdeError::OutOfRange)?,
            Event::F64(f) => f as _,
            ev => {
                return Err(MerdeError::UnexpectedEvent {
                    got: EventType::from(&ev),
                    expected: &[EventType::I64, EventType::U64, EventType::Float],
                })
            }
        };
        Ok(v)
    }
}

impl<'s> Deserialize<'s> for u64 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: u64 = match de.next().await? {
            Event::U64(u) => u,
            Event::I64(i) => i.try_into().map_err(|_| MerdeError::OutOfRange)?,
            Event::F64(f) => f as u64,
            ev => {
                return Err(MerdeError::UnexpectedEvent {
                    got: EventType::from(&ev),
                    expected: &[EventType::U64, EventType::I64, EventType::Float],
                })
            }
        };
        Ok(v)
    }
}

impl<'s> Deserialize<'s> for i32 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: i64 = i64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for u32 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: u64 = u64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for i16 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: i64 = i64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for u16 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: u64 = u64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for i8 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: i64 = i64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for u8 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: u64 = u64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for isize {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: i64 = i64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for usize {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: u64 = u64::deserialize(de).await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange)
    }
}

impl<'s> Deserialize<'s> for bool {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        Ok(de.next().await?.into_bool()?)
    }
}

impl<'s> Deserialize<'s> for f64 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: f64 = match de.next().await? {
            Event::F64(f) => f,
            Event::I64(i) => i as f64,
            Event::U64(u) => u as f64,
            ev => {
                return Err(MerdeError::UnexpectedEvent {
                    got: EventType::from(&ev),
                    expected: &[EventType::Float, EventType::I64, EventType::U64],
                })
            }
        };
        Ok(v)
    }
}

impl<'s> Deserialize<'s> for f32 {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let v: f64 = f64::deserialize(de).await?;
        Ok(v as f32)
    }
}

impl<'s> Deserialize<'s> for String {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let cow: CowStr<'s> = CowStr::deserialize(de).await?;
        Ok(cow.to_string())
    }
}

impl<'s> Deserialize<'s> for CowStr<'s> {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        Ok(de.next().await?.into_str()?)
    }
}

impl<'s> Deserialize<'s> for Cow<'s, str> {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let cow: CowStr<'s> = CowStr::deserialize(de).await?;
        Ok(match cow {
            CowStr::Borrowed(s) => Cow::Borrowed(s),
            CowStr::Owned(s) => Cow::Owned(s.to_string()),
        })
    }
}

impl<'s, T: Deserialize<'s> + 's> Deserialize<'s> for Box<T> {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        let value: T = T::deserialize(de).await?;
        Ok(Box::new(value))
    }
}

impl<'s, T: Deserialize<'s> + 's> Deserialize<'s> for Option<T> {
    async fn deserialize(de: &mut dyn DynDeserializer<'s>) -> Result<Self, MerdeError<'s>> {
        match de.next().await? {
            Event::Null => Ok(None),
            ev => {
                let mut starting_with = de.starting_with(ev);
                let value = T::deserialize(&mut starting_with).await?;
                Ok(Some(value))
            }
        }
    }

    fn from_option(value: Option<Self>, _field_name: CowStr<'s>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(value) => Ok(value),
            None => Ok(None),
        }
    }
}

impl<'s, T: Deserialize<'s>> Deserialize<'s> for Vec<T> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let array_start = de.next().await?.into_array_start()?;
        let mut vec = if let Some(size) = array_start.size_hint {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };

        loop {
            match de.next().await? {
                Event::ArrayEnd => {
                    #[cfg(debug_assertions)]
                    {
                        println!("Stack trace:");
                        let backtrace = std::backtrace::Backtrace::capture();
                        println!("{}", backtrace);
                    }
                    break;
                }
                ev => {
                    let item: T = de.t_starting_with(Some(ev)).await?;
                    vec.push(item);
                }
            }
        }

        Ok(vec)
    }
}

impl<'s, K, V, S> Deserialize<'s> for HashMap<K, V, S>
where
    K: Deserialize<'s> + Eq + Hash,
    V: Deserialize<'s>,
    S: Default + BuildHasher + 's,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_map_start()?;
        let mut map = HashMap::<K, V, S>::default();

        loop {
            match de.next().await? {
                Event::MapEnd => break,
                ev => {
                    let key: K = de.t_starting_with(Some(ev)).await?;
                    let value: V = de.t().await?;
                    map.insert(key, value);
                }
            }
        }

        Ok(map)
    }
}

impl<'s> Deserialize<'s> for Map<'s> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_map_start()?;
        let mut map = Map::new();

        loop {
            match de.next().await? {
                Event::MapEnd => break,
                Event::Str(key) => {
                    let value: Value<'s> = de.t().await?;
                    map.insert(key, value);
                }
                ev => {
                    return Err(MerdeError::UnexpectedEvent {
                        got: EventType::from(&ev),
                        expected: &[EventType::Str, EventType::MapEnd],
                    }
                    .into())
                }
            }
        }

        Ok(map)
    }
}

impl<'s> Deserialize<'s> for Array<'s> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let array_start = de.next().await?.into_array_start()?;
        let mut array = if let Some(size) = array_start.size_hint {
            Array::with_capacity(size)
        } else {
            Array::new()
        };

        loop {
            match de.next().await? {
                Event::ArrayEnd => break,
                ev => {
                    let item: Value<'s> = de.t_starting_with(Some(ev)).await?;
                    array.push(item);
                }
            }
        }

        Ok(array)
    }
}

impl<'s> Deserialize<'s> for Value<'s> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        match de.next().await? {
            Event::I64(i) => Ok(Value::I64(i)),
            Event::U64(u) => Ok(Value::U64(u)),
            Event::F64(f) => Ok(Value::Float(f.into())),
            Event::Str(s) => Ok(Value::Str(s)),
            Event::Bytes(b) => Ok(Value::Bytes(b)),
            Event::Bool(b) => Ok(Value::Bool(b)),
            Event::Null => Ok(Value::Null),
            Event::MapStart(ms) => {
                let mut map = match ms.size_hint {
                    Some(size) => Map::with_capacity(size),
                    None => Map::new(),
                };
                loop {
                    match de.next().await? {
                        Event::MapEnd => break,
                        Event::Str(key) => {
                            let value: Value = de
                                .t_starting_with(None)
                                .with_metastack_resume_point()
                                .await?;
                            map.insert(key, value);
                        }
                        ev => {
                            return Err(MerdeError::UnexpectedEvent {
                                got: EventType::from(&ev),
                                expected: &[EventType::Str, EventType::MapEnd],
                            }
                            .into())
                        }
                    }
                }
                Ok(Value::Map(map))
            }
            Event::ArrayStart(_) => {
                let mut vec = Array::new();
                loop {
                    match de.next().await? {
                        Event::ArrayEnd => break,
                        ev => {
                            let item: Value = de
                                .t_starting_with(Some(ev))
                                .with_metastack_resume_point()
                                .await?;
                            vec.push(item);
                        }
                    }
                }
                Ok(Value::Array(vec))
            }
            ev => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&ev),
                expected: &[
                    EventType::I64,
                    EventType::U64,
                    EventType::Float,
                    EventType::Str,
                    EventType::Bytes,
                    EventType::Bool,
                    EventType::Null,
                    EventType::MapStart,
                    EventType::ArrayStart,
                ],
            }
            .into()),
        }
    }
}

impl<'s, T1> Deserialize<'s> for (T1,)
where
    T1: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1,))
    }
}

impl<'s, T1, T2> Deserialize<'s> for (T1, T2)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2))
    }
}

impl<'s, T1, T2, T3> Deserialize<'s> for (T1, T2, T3)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
    T3: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2, t3))
    }
}

impl<'s, T1, T2, T3, T4> Deserialize<'s> for (T1, T2, T3, T4)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
    T3: Deserialize<'s>,
    T4: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2, t3, t4))
    }
}

impl<'s, T1, T2, T3, T4, T5> Deserialize<'s> for (T1, T2, T3, T4, T5)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
    T3: Deserialize<'s>,
    T4: Deserialize<'s>,
    T5: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2, t3, t4, t5))
    }
}

impl<'s, T1, T2, T3, T4, T5, T6> Deserialize<'s> for (T1, T2, T3, T4, T5, T6)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
    T3: Deserialize<'s>,
    T4: Deserialize<'s>,
    T5: Deserialize<'s>,
    T6: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        let t6 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2, t3, t4, t5, t6))
    }
}

impl<'s, T1, T2, T3, T4, T5, T6, T7> Deserialize<'s> for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
    T3: Deserialize<'s>,
    T4: Deserialize<'s>,
    T5: Deserialize<'s>,
    T6: Deserialize<'s>,
    T7: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        let t6 = de.t().await?;
        let t7 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2, t3, t4, t5, t6, t7))
    }
}

impl<'s, T1, T2, T3, T4, T5, T6, T7, T8> Deserialize<'s> for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: Deserialize<'s>,
    T2: Deserialize<'s>,
    T3: Deserialize<'s>,
    T4: Deserialize<'s>,
    T5: Deserialize<'s>,
    T6: Deserialize<'s>,
    T7: Deserialize<'s>,
    T8: Deserialize<'s>,
{
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.next().await?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        let t6 = de.t().await?;
        let t7 = de.t().await?;
        let t8 = de.t().await?;
        de.next().await?.into_array_end()?;
        Ok((t1, t2, t3, t4, t5, t6, t7, t8))
    }
}

#[cfg(test)]
mod tests;
