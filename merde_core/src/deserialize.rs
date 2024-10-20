use std::{
    any::TypeId,
    borrow::Cow,
    collections::HashMap,
    future::Future,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::{
    Array, CowBytes, CowStr, IntoStatic, Map, MerdeError, StackInfo, Value, WithLifetime,
    NEXT_FUTURE,
};

#[derive(Debug)]
pub enum Event<'s> {
    I64(i64),
    U64(u64),
    Float(f64),
    Str(CowStr<'s>),
    Bytes(CowBytes<'s>),
    Bool(bool),
    Null,
    MapStart,
    MapEnd,
    ArrayStart(ArrayStart),
    ArrayEnd,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventType {
    I64,
    U64,
    Float,
    Str,
    Bytes,
    Bool,
    Null,
    MapStart,
    MapEnd,
    ArrayStart,
    ArrayEnd,
}

impl From<&Event<'_>> for EventType {
    fn from(event: &Event<'_>) -> Self {
        match event {
            Event::I64(_) => EventType::I64,
            Event::U64(_) => EventType::U64,
            Event::Float(_) => EventType::Float,
            Event::Str(_) => EventType::Str,
            Event::Bytes(_) => EventType::Bytes,
            Event::Bool(_) => EventType::Bool,
            Event::Null => EventType::Null,
            Event::MapStart => EventType::MapStart,
            Event::MapEnd => EventType::MapEnd,
            Event::ArrayStart(_) => EventType::ArrayStart,
            Event::ArrayEnd => EventType::ArrayEnd,
        }
    }
}

#[derive(Debug)]
pub struct ArrayStart {
    pub size_hint: Option<usize>,
}

impl<'s> Event<'s> {
    pub fn into_i64(self) -> Result<i64, MerdeError<'static>> {
        match self {
            Event::I64(i) => Ok(i),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::I64],
            }),
        }
    }

    pub fn into_u64(self) -> Result<u64, MerdeError<'static>> {
        match self {
            Event::U64(u) => Ok(u),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::U64],
            }),
        }
    }

    pub fn into_f64(self) -> Result<f64, MerdeError<'static>> {
        match self {
            Event::Float(f) => Ok(f),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Float],
            }),
        }
    }

    pub fn into_str(self) -> Result<CowStr<'s>, MerdeError<'static>> {
        match self {
            Event::Str(s) => Ok(s),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Str],
            }),
        }
    }

    pub fn into_bytes(self) -> Result<CowBytes<'s>, MerdeError<'static>> {
        match self {
            Event::Bytes(b) => Ok(b),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Bytes],
            }),
        }
    }

    pub fn into_bool(self) -> Result<bool, MerdeError<'static>> {
        match self {
            Event::Bool(b) => Ok(b),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Bool],
            }),
        }
    }

    pub fn into_null(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::Null => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::Null],
            }),
        }
    }

    pub fn into_map_start(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::MapStart => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::MapStart],
            }),
        }
    }

    pub fn into_map_end(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::MapEnd => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::MapEnd],
            }),
        }
    }

    pub fn into_array_start(self) -> Result<ArrayStart, MerdeError<'static>> {
        match self {
            Event::ArrayStart(array_start) => Ok(array_start),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::ArrayStart],
            }),
        }
    }

    pub fn into_array_end(self) -> Result<(), MerdeError<'static>> {
        match self {
            Event::ArrayEnd => Ok(()),
            _ => Err(MerdeError::UnexpectedEvent {
                got: EventType::from(&self),
                expected: &[EventType::ArrayEnd],
            }),
        }
    }
}

pub trait Deserializer<'s>: std::fmt::Debug {
    type Error<'es>: From<MerdeError<'es>>;

    /// Get the next event from the deserializer.
    #[doc(hidden)]
    fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>>;

    /// Deserialize a value of type `T`.
    #[doc(hidden)]
    #[allow(async_fn_in_trait)]
    async fn t<T: Deserialize<'s>>(&mut self) -> Result<T, Self::Error<'s>> {
        self.t_starting_with(None).await
    }

    /// Deserialize a value of type `T`, using the given event as the first event.
    #[doc(hidden)]
    #[allow(async_fn_in_trait)]
    async fn t_starting_with<T: Deserialize<'s>>(
        &mut self,
        starter: Option<Event<'s>>,
    ) -> Result<T, Self::Error<'s>>;

    /// Return a boxed version of `Self::t_starting_with`, useful to avoid
    /// future sizes becoming infinite, for example when deserializing Value,
    /// etc.
    #[doc(hidden)]
    fn t_starting_with_boxed<'d, T: Deserialize<'s> + 'd>(
        &'d mut self,
        starter: Option<Event<'s>>,
    ) -> Pin<Box<dyn Future<Output = Result<T, Self::Error<'s>>> + 'd>>
    where
        's: 'd,
    {
        // TODO: cache in a thread-local, or, more simply, in a deserialization context?
        let stack_info = StackInfo::get();

        let fut = self.t_starting_with(starter);
        Box::pin(async move {
            // TODO: 8K is not one-size-fits-all
            if stack_info.left() < 8 * 1024 {
                // this is probably not actually on the stack because we're in a boxed future
                let mut result: Option<Result<T, Self::Error<'s>>> = None;

                // first turn it into a trait object
                let background_fut: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async {
                    result = Some(fut.await);
                });

                let background_fut: Pin<Box<dyn Future<Output = ()> + 'static>> = unsafe {
                    // # Safety: this isn't actually 'static, it's "valid for the synchronous
                    // call to deserialize".
                    // todo: make sure that this is actually the case by handling panics and
                    // clearing thread-locals.
                    std::mem::transmute(background_fut)
                };

                NEXT_FUTURE.with_borrow_mut(|next_future| *next_future = Some(background_fut));
                ReturnPendingOnce::new().await;
                result.unwrap()
            } else {
                fut.await
            }
        })
    }

    fn deserialize<T: Deserialize<'s>>(&mut self) -> Result<T, Self::Error<'s>> {
        const VTABLE: RawWakerVTable = RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
        let w = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&w);
        let first_fut = self.t_starting_with(None);
        let mut first_fut = std::pin::pin!(first_fut);

        match first_fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => res,
            _ => {
                // oh boy. okay.
                let mut stack = vec![];

                'crimes: loop {
                    let mut fut = NEXT_FUTURE
                        .with_borrow_mut(|next_fut| next_fut.take())
                        .expect("NEXT_FUTURE must've been set before returning Poll::Pending");
                    match Pin::new(&mut fut).poll(&mut cx) {
                        Poll::Ready(_) => break 'crimes,
                        Poll::Pending => {
                            stack.push(fut);
                        }
                    }
                }

                while let Some(mut fut) = stack.pop() {
                    match Pin::new(&mut fut).poll(&mut cx) {
                        Poll::Ready(_) => {
                            // cool let's keep going
                        }
                        Poll::Pending => {
                            unreachable!("I'm sorry you really only get to ask for more stack once")
                        }
                    }
                }

                match first_fut.poll(&mut cx) {
                    Poll::Ready(res) => res,
                    Poll::Pending => {
                        unreachable!("Like I said, you really only get to ask for more stack once")
                    }
                }
            }
        }
    }

    /// Deserialize a value of type `T` and return its static variant
    /// e.g. (CowStr<'static>, etc.)
    fn deserialize_owned<T>(&mut self) -> Result<T, Self::Error<'s>>
    where
        T: 'static,
        T: WithLifetime<'s>,
        <T as WithLifetime<'s>>::Lifetimed: Deserialize<'s> + IntoStatic<Output = T>,
    {
        self.deserialize()
            .map(|t: <T as WithLifetime<'s>>::Lifetimed| t.into_static())
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
    #[allow(async_fn_in_trait)]
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized;

    fn from_option(value: Option<Self>, field_name: CowStr<'s>) -> Result<Self, MerdeError<'s>> {
        match value {
            Some(value) => Ok(value),
            None => Err(MerdeError::MissingProperty(field_name)),
        }
    }
}

pub trait DeserializeOwned: Sized {
    fn deserialize_owned<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized;
}

impl<T> DeserializeOwned for T
where
    T: for<'s> WithLifetime<'s> + 'static,
    for<'s> <T as WithLifetime<'s>>::Lifetimed: Deserialize<'s> + IntoStatic<Output = T>,
{
    fn deserialize_owned<'s, D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        de.deserialize_owned()
    }
}

impl<'s> Deserialize<'s> for i64 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = match de.next()? {
            Event::I64(i) => i,
            Event::U64(u) => u.try_into().map_err(|_| MerdeError::OutOfRange)?,
            Event::Float(f) => f as _,
            ev => {
                return Err(MerdeError::UnexpectedEvent {
                    got: EventType::from(&ev),
                    expected: &[EventType::I64, EventType::U64, EventType::Float],
                }
                .into())
            }
        };
        Ok(v)
    }
}

impl<'s> Deserialize<'s> for u64 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: u64 = match de.next()? {
            Event::U64(u) => u,
            Event::I64(i) => i.try_into().map_err(|_| MerdeError::OutOfRange)?,
            Event::Float(f) => f as u64,
            ev => {
                return Err(MerdeError::UnexpectedEvent {
                    got: EventType::from(&ev),
                    expected: &[EventType::U64, EventType::I64, EventType::Float],
                }
                .into())
            }
        };
        Ok(v)
    }
}

impl<'s> Deserialize<'s> for i32 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for u32 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: u64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for i16 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for u16 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: u64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for i8 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for u8 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: u64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for isize {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: i64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for usize {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: u64 = de.t().await?;
        v.try_into().map_err(|_| MerdeError::OutOfRange.into())
    }
}

impl<'s> Deserialize<'s> for bool {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        Ok(de.next()?.into_bool()?)
    }
}

impl<'s> Deserialize<'s> for f64 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: f64 = match de.next()? {
            Event::Float(f) => f,
            Event::I64(i) => i as f64,
            Event::U64(u) => u as f64,
            ev => {
                return Err(MerdeError::UnexpectedEvent {
                    got: EventType::from(&ev),
                    expected: &[EventType::Float, EventType::I64, EventType::U64],
                }
                .into())
            }
        };
        Ok(v)
    }
}

impl<'s> Deserialize<'s> for f32 {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let v: f64 = de.t().await?;
        Ok(v as f32)
    }
}

impl<'s> Deserialize<'s> for String {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let cow: CowStr<'s> = de.t().await?;
        Ok(cow.to_string())
    }
}

impl<'s> Deserialize<'s> for CowStr<'s> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        Ok(de.next()?.into_str()?)
    }
}

impl<'s> Deserialize<'s> for Cow<'s, str> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        let cow: CowStr<'s> = de.t().await?;
        Ok(match cow {
            CowStr::Borrowed(s) => Cow::Borrowed(s),
            CowStr::Owned(s) => Cow::Owned(s.to_string()),
        })
    }
}

impl<'s, T: Deserialize<'s>> Deserialize<'s> for Option<T> {
    async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
    where
        D: Deserializer<'s> + ?Sized,
    {
        match de.next()? {
            Event::Null => Ok(None),
            ev => {
                let value = de.t_starting_with(Some(ev)).await?;
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
        let array_start = de.next()?.into_array_start()?;
        let mut vec = if let Some(size) = array_start.size_hint {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };

        loop {
            match de.next()? {
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
        de.next()?.into_map_start()?;
        let mut map = HashMap::<K, V, S>::default();

        loop {
            match de.next()? {
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
        de.next()?.into_map_start()?;
        let mut map = Map::new();

        loop {
            match de.next()? {
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
        let array_start = de.next()?.into_array_start()?;
        let mut array = if let Some(size) = array_start.size_hint {
            Array::with_capacity(size)
        } else {
            Array::new()
        };

        loop {
            match de.next()? {
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
        match de.next()? {
            Event::I64(i) => Ok(Value::I64(i)),
            Event::U64(u) => Ok(Value::U64(u)),
            Event::Float(f) => Ok(Value::Float(f.into())),
            Event::Str(s) => Ok(Value::Str(s)),
            Event::Bytes(b) => Ok(Value::Bytes(b)),
            Event::Bool(b) => Ok(Value::Bool(b)),
            Event::Null => Ok(Value::Null),
            Event::MapStart => {
                let mut map = Map::new();
                loop {
                    match de.next()? {
                        Event::MapEnd => break,
                        Event::Str(key) => {
                            let value: Value = de.t_starting_with_boxed(None).await?;
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
                    match de.next()? {
                        Event::ArrayEnd => break,
                        ev => {
                            let item: Value = de.t_starting_with_boxed(Some(ev)).await?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        let t6 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        let t6 = de.t().await?;
        let t7 = de.t().await?;
        de.next()?.into_array_end()?;
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
        de.next()?.into_array_start()?;
        let t1 = de.t().await?;
        let t2 = de.t().await?;
        let t3 = de.t().await?;
        let t4 = de.t().await?;
        let t5 = de.t().await?;
        let t6 = de.t().await?;
        let t7 = de.t().await?;
        let t8 = de.t().await?;
        de.next()?.into_array_end()?;
        Ok((t1, t2, t3, t4, t5, t6, t7, t8))
    }
}

struct ReturnPendingOnce {
    polled: bool,
}

impl ReturnPendingOnce {
    fn new() -> Self {
        Self { polled: false }
    }
}

impl Future for ReturnPendingOnce {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.polled {
            Poll::Ready(())
        } else {
            this.polled = true;
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests;
