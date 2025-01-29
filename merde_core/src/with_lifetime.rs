use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use crate::{CowStr, Value};

/// Allow instantiating a type with a lifetime parameter, which in
/// turn lets us require `Deserialize<'s>` for `CowStr<'s>` for
/// example, even when `CowStr<'s>` is erased behind a `T`.
///
/// See <https://github.com/bearcove/merde/pull/60> for details
pub trait WithLifetime<'s> {
    type Lifetimed: 's;
}

macro_rules! impl_with_lifetime {
    ($($struct_name:ident $(<$lifetime:lifetime>)?),* $(,)?) => {
        $(
            impl_with_lifetime!(@inner $struct_name $(<$lifetime>)?);
        )*
    };

    (@inner $struct_name:ident <$lifetime:lifetime>) => {
        impl<$lifetime, 'instantiated_lifetime> WithLifetime<'instantiated_lifetime>
            for $struct_name<$lifetime>
        {
            type Lifetimed = $struct_name<'instantiated_lifetime>;
        }
    };

    (@inner $struct_name:ident) => {
        impl<'s> WithLifetime<'s> for $struct_name {
            type Lifetimed = $struct_name;
        }
    };
}

impl<'s, B: ToOwned + ?Sized + 's> WithLifetime<'s> for Cow<'_, B> {
    type Lifetimed = Cow<'s, B>;
}

impl<'s> WithLifetime<'s> for &str {
    type Lifetimed = &'s str;
}

impl_with_lifetime!(
    Value<'s>,
    CowStr<'s>,
    String,
    u128,
    u64,
    u32,
    u16,
    u8,
    i128,
    i64,
    i32,
    i16,
    i8,
    bool,
    char,
    usize,
    isize,
    f32,
    f64,
);

impl WithLifetime<'_> for () {
    type Lifetimed = ();
}

impl<'s, T> WithLifetime<'s> for Option<T>
where
    T: WithLifetime<'s>,
{
    type Lifetimed = Option<T::Lifetimed>;
}

impl<'s, T> WithLifetime<'s> for Vec<T>
where
    T: WithLifetime<'s>,
{
    type Lifetimed = Vec<T::Lifetimed>;
}

impl<'s, T> WithLifetime<'s> for Arc<T>
where
    T: WithLifetime<'s>,
{
    type Lifetimed = Arc<T::Lifetimed>;
}

impl<'s, T> WithLifetime<'s> for VecDeque<T>
where
    T: WithLifetime<'s>,
{
    type Lifetimed = VecDeque<T::Lifetimed>;
}

impl<'s, T> WithLifetime<'s> for HashSet<T>
where
    T: WithLifetime<'s>,
{
    type Lifetimed = HashSet<T::Lifetimed>;
}

impl<'s, K, V, S> WithLifetime<'s> for HashMap<K, V, S>
where
    S: 's,
    K: WithLifetime<'s>,
    V: WithLifetime<'s>,
{
    type Lifetimed = HashMap<K::Lifetimed, V::Lifetimed, S>;
}

impl<'s, T1: WithLifetime<'s>> WithLifetime<'s> for (T1,) {
    type Lifetimed = (T1::Lifetimed,);
}

impl<'s, T1: WithLifetime<'s>, T2: WithLifetime<'s>> WithLifetime<'s> for (T1, T2) {
    type Lifetimed = (T1::Lifetimed, T2::Lifetimed);
}

impl<'s, T1: WithLifetime<'s>, T2: WithLifetime<'s>, T3: WithLifetime<'s>> WithLifetime<'s>
    for (T1, T2, T3)
{
    type Lifetimed = (T1::Lifetimed, T2::Lifetimed, T3::Lifetimed);
}

impl<
        's,
        T1: WithLifetime<'s>,
        T2: WithLifetime<'s>,
        T3: WithLifetime<'s>,
        T4: WithLifetime<'s>,
    > WithLifetime<'s> for (T1, T2, T3, T4)
{
    type Lifetimed = (T1::Lifetimed, T2::Lifetimed, T3::Lifetimed, T4::Lifetimed);
}

impl<
        's,
        T1: WithLifetime<'s>,
        T2: WithLifetime<'s>,
        T3: WithLifetime<'s>,
        T4: WithLifetime<'s>,
        T5: WithLifetime<'s>,
    > WithLifetime<'s> for (T1, T2, T3, T4, T5)
{
    type Lifetimed = (
        T1::Lifetimed,
        T2::Lifetimed,
        T3::Lifetimed,
        T4::Lifetimed,
        T5::Lifetimed,
    );
}

impl<
        's,
        T1: WithLifetime<'s>,
        T2: WithLifetime<'s>,
        T3: WithLifetime<'s>,
        T4: WithLifetime<'s>,
        T5: WithLifetime<'s>,
        T6: WithLifetime<'s>,
    > WithLifetime<'s> for (T1, T2, T3, T4, T5, T6)
{
    type Lifetimed = (
        T1::Lifetimed,
        T2::Lifetimed,
        T3::Lifetimed,
        T4::Lifetimed,
        T5::Lifetimed,
        T6::Lifetimed,
    );
}

impl<
        's,
        T1: WithLifetime<'s>,
        T2: WithLifetime<'s>,
        T3: WithLifetime<'s>,
        T4: WithLifetime<'s>,
        T5: WithLifetime<'s>,
        T6: WithLifetime<'s>,
        T7: WithLifetime<'s>,
    > WithLifetime<'s> for (T1, T2, T3, T4, T5, T6, T7)
{
    type Lifetimed = (
        T1::Lifetimed,
        T2::Lifetimed,
        T3::Lifetimed,
        T4::Lifetimed,
        T5::Lifetimed,
        T6::Lifetimed,
        T7::Lifetimed,
    );
}

impl<
        's,
        T1: WithLifetime<'s>,
        T2: WithLifetime<'s>,
        T3: WithLifetime<'s>,
        T4: WithLifetime<'s>,
        T5: WithLifetime<'s>,
        T6: WithLifetime<'s>,
        T7: WithLifetime<'s>,
        T8: WithLifetime<'s>,
    > WithLifetime<'s> for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    type Lifetimed = (
        T1::Lifetimed,
        T2::Lifetimed,
        T3::Lifetimed,
        T4::Lifetimed,
        T5::Lifetimed,
        T6::Lifetimed,
        T7::Lifetimed,
        T8::Lifetimed,
    );
}
