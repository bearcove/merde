use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::hash::Hash;

/// Allow turning a value into an "owned" variant, which can then be
/// returned, moved, etc.
///
/// This usually involves allocating buffers for `Cow<'a, str>`, etc.
pub trait IntoStatic {
    /// The "owned" variant of the type. For `Cow<'a, str>`, this is `Cow<'static, str>`, for example.
    type Output: 'static;

    /// Turns the value into an "owned" variant, which can then be returned, moved, etc.
    ///
    /// This allocates, for all but the most trivial types.
    fn into_static(self) -> Self::Output;
}

impl<'a, T> IntoStatic for Cow<'a, T>
where
    T: ToOwned + ?Sized + 'static,
{
    type Output = Cow<'static, T>;

    fn into_static(self) -> Self::Output {
        match self {
            Cow::Borrowed(b) => Cow::Owned(b.to_owned()),
            Cow::Owned(o) => Cow::Owned(o),
        }
    }
}

macro_rules! impl_into_static_passthru {
    (($($ty:ty),+)) => {
        $(
            impl IntoStatic for $ty {
                type Output = $ty;

                #[inline(always)]
                fn into_static(self) -> Self::Output {
                    self
                }
            }
        )+
    };
}

impl_into_static_passthru!((
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, bool, String
));

impl<T: IntoStatic> IntoStatic for Option<T> {
    type Output = Option<T::Output>;

    fn into_static(self) -> Self::Output {
        self.map(|v| v.into_static())
    }
}

impl<T: IntoStatic> IntoStatic for Vec<T> {
    type Output = Vec<T::Output>;

    fn into_static(self) -> Self::Output {
        self.into_iter().map(|v| v.into_static()).collect()
    }
}

impl<K, V> IntoStatic for HashMap<K, V>
where
    K: IntoStatic + Eq + Hash,
    V: IntoStatic,
    K::Output: Eq + Hash,
{
    type Output = HashMap<K::Output, V::Output>;

    fn into_static(self) -> Self::Output {
        self.into_iter()
            .map(|(k, v)| (k.into_static(), v.into_static()))
            .collect()
    }
}

impl<T: IntoStatic> IntoStatic for HashSet<T>
where
    T::Output: Eq + Hash,
{
    type Output = HashSet<T::Output>;

    fn into_static(self) -> Self::Output {
        self.into_iter().map(|v| v.into_static()).collect()
    }
}

impl<T: IntoStatic> IntoStatic for VecDeque<T> {
    type Output = VecDeque<T::Output>;

    fn into_static(self) -> Self::Output {
        self.into_iter().map(|v| v.into_static()).collect()
    }
}
