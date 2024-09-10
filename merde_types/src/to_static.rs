use std::hash::Hash;

/// Allow turning a value into an "owned" variant, which can then be
/// returned, moved, etc.
///
/// This usually involves allocating buffers for `Cow<'a, str>`, etc.
pub trait ToStatic {
    /// The "owned" variant of the type. For `Cow<'a, str>`, this is `Cow<'static, str>`, for example.
    type Output: 'static;

    /// Turns the value into an "owned" variant, which can then be returned, moved, etc.
    ///
    /// This allocates, for all but the most trivial types.
    fn to_static(&self) -> Self::Output;
}

impl<'a, T> ToStatic for Cow<'a, T>
where
    T: ToOwned + ?Sized + 'static,
{
    type Output = Cow<'static, T>;

    fn to_static(&self) -> Self::Output {
        match self.clone() {
            Cow::Borrowed(b) => Cow::Owned(b.to_owned()),
            Cow::Owned(o) => Cow::Owned(o),
        }
    }
}

impl ToStatic for u8 {
    type Output = u8;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for u16 {
    type Output = u16;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for u32 {
    type Output = u32;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for u64 {
    type Output = u64;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i8 {
    type Output = i8;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i16 {
    type Output = i16;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i32 {
    type Output = i32;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for i64 {
    type Output = i64;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for usize {
    type Output = usize;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for isize {
    type Output = isize;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for bool {
    type Output = bool;

    fn to_static(&self) -> Self::Output {
        *self
    }
}

impl ToStatic for String {
    type Output = String;

    fn to_static(&self) -> Self::Output {
        self.clone()
    }
}

impl<T: ToStatic> ToStatic for Option<T> {
    type Output = Option<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.as_ref().map(|v| v.to_static())
    }
}

impl<T: ToStatic> ToStatic for Vec<T> {
    type Output = Vec<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter().map(|v| v.to_static()).collect()
    }
}

impl<K, V> ToStatic for HashMap<K, V>
where
    K: ToStatic + Eq + Hash,
    V: ToStatic,
    K::Output: Eq + Hash,
{
    type Output = HashMap<K::Output, V::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter()
            .map(|(k, v)| (k.to_static(), v.to_static()))
            .collect()
    }
}

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
};

impl<T: ToStatic> ToStatic for HashSet<T>
where
    T::Output: Eq + Hash,
{
    type Output = HashSet<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter().map(|v| v.to_static()).collect()
    }
}

impl<T: ToStatic> ToStatic for VecDeque<T> {
    type Output = VecDeque<T::Output>;

    fn to_static(&self) -> Self::Output {
        self.iter().map(|v| v.to_static()).collect()
    }
}
