use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
};

use crate::CowStr;

/// Allow instantiating a type with a lifetime parameter, which in
/// turn lets us require `ValueDeserialize<'s>` for `CowStr<'s>` for
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

impl<'a, 's, B: ToOwned + ?Sized + 's> WithLifetime<'s> for Cow<'a, B> {
    type Lifetimed = Cow<'s, B>;
}

impl<'a, 's> WithLifetime<'s> for &'a str {
    type Lifetimed = &'s str;
}

impl_with_lifetime!(
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
    f32,
    f64,
    usize,
    isize,
);

impl<'s> WithLifetime<'s> for () {
    type Lifetimed = ();
}

impl<'s, T> WithLifetime<'s> for Vec<T>
where
    T: WithLifetime<'s>,
{
    type Lifetimed = Vec<T::Lifetimed>;
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

impl<'s, K, V> WithLifetime<'s> for HashMap<K, V>
where
    K: WithLifetime<'s>,
    V: WithLifetime<'s>,
{
    type Lifetimed = HashMap<K::Lifetimed, V::Lifetimed>;
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
