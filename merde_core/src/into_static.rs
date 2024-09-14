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

    #[inline(always)]
    fn into_static(self) -> Self::Output {
        match self {
            Cow::Borrowed(b) => Cow::Owned(b.to_owned()),
            Cow::Owned(o) => Cow::Owned(o),
        }
    }
}

macro_rules! impl_into_static_passthru {
    ($($ty:ty),+) => {
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

impl_into_static_passthru!(
    String, u128, u64, u32, u16, u8, i128, i64, i32, i16, i8, bool, char, f32, f64, usize, isize
);

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

impl<T1: IntoStatic> IntoStatic for (T1,) {
    type Output = (T1::Output,);

    fn into_static(self) -> Self::Output {
        (self.0.into_static(),)
    }
}

impl<T1: IntoStatic, T2: IntoStatic> IntoStatic for (T1, T2) {
    type Output = (T1::Output, T2::Output);

    fn into_static(self) -> Self::Output {
        (self.0.into_static(), self.1.into_static())
    }
}

impl<T1: IntoStatic, T2: IntoStatic, T3: IntoStatic> IntoStatic for (T1, T2, T3) {
    type Output = (T1::Output, T2::Output, T3::Output);

    fn into_static(self) -> Self::Output {
        (
            self.0.into_static(),
            self.1.into_static(),
            self.2.into_static(),
        )
    }
}

impl<T1: IntoStatic, T2: IntoStatic, T3: IntoStatic, T4: IntoStatic> IntoStatic
    for (T1, T2, T3, T4)
{
    type Output = (T1::Output, T2::Output, T3::Output, T4::Output);

    fn into_static(self) -> Self::Output {
        (
            self.0.into_static(),
            self.1.into_static(),
            self.2.into_static(),
            self.3.into_static(),
        )
    }
}

impl<T1: IntoStatic, T2: IntoStatic, T3: IntoStatic, T4: IntoStatic, T5: IntoStatic> IntoStatic
    for (T1, T2, T3, T4, T5)
{
    type Output = (T1::Output, T2::Output, T3::Output, T4::Output, T5::Output);

    fn into_static(self) -> Self::Output {
        (
            self.0.into_static(),
            self.1.into_static(),
            self.2.into_static(),
            self.3.into_static(),
            self.4.into_static(),
        )
    }
}

impl<
        T1: IntoStatic,
        T2: IntoStatic,
        T3: IntoStatic,
        T4: IntoStatic,
        T5: IntoStatic,
        T6: IntoStatic,
    > IntoStatic for (T1, T2, T3, T4, T5, T6)
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
    );

    fn into_static(self) -> Self::Output {
        (
            self.0.into_static(),
            self.1.into_static(),
            self.2.into_static(),
            self.3.into_static(),
            self.4.into_static(),
            self.5.into_static(),
        )
    }
}

impl<
        T1: IntoStatic,
        T2: IntoStatic,
        T3: IntoStatic,
        T4: IntoStatic,
        T5: IntoStatic,
        T6: IntoStatic,
        T7: IntoStatic,
    > IntoStatic for (T1, T2, T3, T4, T5, T6, T7)
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
    );

    fn into_static(self) -> Self::Output {
        (
            self.0.into_static(),
            self.1.into_static(),
            self.2.into_static(),
            self.3.into_static(),
            self.4.into_static(),
            self.5.into_static(),
            self.6.into_static(),
        )
    }
}

impl<
        T1: IntoStatic,
        T2: IntoStatic,
        T3: IntoStatic,
        T4: IntoStatic,
        T5: IntoStatic,
        T6: IntoStatic,
        T7: IntoStatic,
        T8: IntoStatic,
    > IntoStatic for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
    );

    fn into_static(self) -> Self::Output {
        (
            self.0.into_static(),
            self.1.into_static(),
            self.2.into_static(),
            self.3.into_static(),
            self.4.into_static(),
            self.5.into_static(),
            self.6.into_static(),
            self.7.into_static(),
        )
    }
}
