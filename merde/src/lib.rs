#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "json")]
pub use merde_json as json;

#[cfg(feature = "yaml")]
pub use merde_yaml as yaml;

#[cfg(feature = "core")]
pub use merde_core::*;

#[doc(hidden)]
#[cfg(feature = "deserialize")]
#[macro_export]
macro_rules! impl_deserialize {
    // owned tuple struct (transparent)
    (struct $struct_name:ident transparent) => {
        #[automatically_derived]
        impl<'s> $crate::Deserialize<'s> for $struct_name {
            #[inline(always)]
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<'s>) -> Result<Self, $crate::MerdeError<'s>> {
                use $crate::DynDeserializerExt;

                Ok(Self(__de.t().await?))
            }
        }
    };

    // lifetimed tuple struct (transparent)
    (struct $struct_name:ident <$s:lifetime> transparent) => {
        #[automatically_derived]
        impl<$s> $crate::Deserialize<$s> for $struct_name<$s> {
            #[inline(always)]
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<'s>) -> Result<Self, $crate::MerdeError<'s>> {
                use $crate::DynDeserializerExt;

                Ok(Self(__de.t().await?))
            }
        }
    };

    // owned struct
    (struct $struct_name:ident { $($field:ident),* }) => {
        $crate::impl_deserialize! {
            struct $struct_name { $($field),* } via $crate::DefaultDeserOpinions
        }
    };
    (struct $struct_name:ident { $($field:ident),* } via $opinions:expr) => {
        #[automatically_derived]
        impl<'s> $crate::Deserialize<'s> for $struct_name {
            #[inline(always)]
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<'s>) -> Result<Self, $crate::MerdeError<'s>> {
                #![allow(unreachable_code)]
                use $crate::{DynDeserializerExt, DeserOpinions};

                let __opinions = $opinions;
                __de.next().await?.into_map_start()?;

                $(
                    let mut $field = $crate::none_of(|i: $struct_name| i.$field);
                )+

                loop {
                    match __de.next().await? {
                        $crate::Event::MapEnd => break,
                        $crate::Event::Str(__key) => {
                            let __key = __opinions.map_key_name(__key);
                            match __key.as_ref() {
                                $(stringify!($field) => {
                                    $field = Some(__de.t().await?);
                                })*
                                _ => {
                                    if __opinions.deny_unknown_fields() {
                                        return Err($crate::MerdeError::UnknownProperty(__key).into());
                                    }
                                }
                            }
                        }
                        ev => {
                            return Err($crate::MerdeError::UnexpectedEvent {
                                got: $crate::EventType::from(&ev),
                                expected: &[$crate::EventType::Str, $crate::EventType::MapEnd],
                                help: Some(format!("While deserializing {}", stringify!($struct_name))),
                            }
                            .into())
                        }
                    }
                }

                Ok($struct_name {
                    $($field: {
                        if $field.is_none() {
                            let __slot = $crate::FieldSlot::new(&mut $field);
                            __opinions.default_field_value(stringify!($field), __slot);
                        }
                        $crate::Deserialize::from_option($field, stringify!($field).into())?
                    },)+
                })
            }
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* }) => {
        $crate::impl_deserialize! {
            struct $struct_name <$lifetime> { $($field),* } via $crate::DefaultDeserOpinions
        }
    };
    (struct $struct_name:ident <$s:lifetime> { $($field:ident),* } via $opinions:expr) => {
        #[automatically_derived]
        impl<$s> $crate::Deserialize<$s> for $struct_name<$s> {
            #[inline(always)]
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<$s>) -> Result<Self, $crate::MerdeError<$s>> {
                #![allow(unreachable_code)]
                use $crate::{DeserOpinions, DynDeserializerExt};

                let __opinions = $opinions;
                __de.next().await?.into_map_start()?;

                $(
                    let mut $field = $crate::none_of(|i: $struct_name<$s>| i.$field);
                )+

                loop {
                    match __de.next().await? {
                        $crate::Event::MapEnd => break,
                        $crate::Event::Str(__key) => {
                            let __key = __opinions.map_key_name(__key);
                            match __key.as_ref() {
                                $(stringify!($field) => {
                                    $field = Some(__de.t().await?);
                                })*
                                _ => {
                                    if __opinions.deny_unknown_fields() {
                                        return Err($crate::MerdeError::UnknownProperty(__key).into());
                                    }
                                }
                            }
                        }
                        ev => {
                            return Err($crate::MerdeError::UnexpectedEvent {
                                got: $crate::EventType::from(&ev),
                                expected: &[$crate::EventType::Str, $crate::EventType::MapEnd],
                                help: Some(format!("While deserializing {}", stringify!($struct_name))),
                            }
                            .into())
                        }
                    }
                }

                Ok($struct_name {
                    $($field: {
                        if $field.is_none() {
                            let __slot = $crate::FieldSlot::new(&mut $field);
                            __opinions.default_field_value(stringify!($field), __slot);
                        }
                        $crate::Deserialize::from_option($field, stringify!($field).into())?
                    },)+
                })
            }
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<'s> $crate::Deserialize<'s> for $enum_name {
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<'s>) -> Result<Self, $crate::MerdeError<'s>> {
                #[allow(unused_imports)]
                use $crate::{MerdeError, DynDeserializerExt};

                __de.next().await?.into_map_start()?;
                let key = __de.next().await?.into_str()?;
                match key.as_ref() {
                    $($variant_str => {
                        let value = __de.t().await?;
                        __de.next().await?.into_map_end()?;
                        Ok($enum_name::$variant(value))
                    },)*
                    _ => Err(MerdeError::UnknownProperty(key).into()),
                }
            }
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::Deserialize<$lifetime> for $enum_name<$lifetime> {
            #[inline(always)]
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<$lifetime>) -> Result<Self, $crate::MerdeError<$lifetime>> {
                #[allow(unused_imports)]
                use $crate::{MerdeError, DynDeserializerExt};

                __de.next().await?.into_map_start()?;
                let key = __de.next().await?.into_str()?;
                match key.as_ref() {
                    $($variant_str => {
                        let value = __de.t().await?;
                        __de.next().await?.into_map_end()?;
                        Ok($enum_name::$variant(value))
                    },)*
                    _ => Err(MerdeError::UnknownProperty(key).into()),
                }
            }
        }
    };

    // owned enum (externally tagged, string-like)
    (enum $enum_name:ident string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<'s> $crate::Deserialize<'s> for $enum_name {
            async fn deserialize(__de: &mut dyn $crate::DynDeserializer<'s>) -> Result<Self, $crate::MerdeError<'s>> {
                #[allow(unused_imports)]
                use $crate::MerdeError;
                use $crate::DynDeserializerExt;

                let s = __de.next().await?.into_str()?;
                match s.as_ref() {
                    $($variant_str => Ok($enum_name::$variant),)*
                    _ => Err(MerdeError::UnknownProperty(s).into()),
                }
            }
        }
    };
}

#[doc(hidden)]
#[cfg(not(feature = "deserialize"))]
#[macro_export]
macro_rules! impl_deserialize {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "core")]
macro_rules! impl_into_static {
    // owned tuple struct (transparent)
    (struct $struct_name:ident transparent) => {
        #[automatically_derived]
        impl $crate::IntoStatic for $struct_name {
            type Output = $struct_name;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                self
            }
        }
    };

    // lifetimed tuple struct (transparent)
    (struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        #[automatically_derived]
        impl<$lifetime> $crate::IntoStatic for $struct_name<$lifetime> {
            type Output = $struct_name<'static>;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                $struct_name(self.0.into_static())
            }
        }
    };

    // owned struct
    (struct $struct_name:ident { $($field:ident),* } $($rest:tt)*) => {
        #[automatically_derived]
        impl $crate::IntoStatic for $struct_name {
            type Output = $struct_name;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                self
            }
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* } $($rest:tt)*) => {
        #[automatically_derived]
        impl<$lifetime> $crate::IntoStatic for $struct_name<$lifetime> {
            type Output = $struct_name<'static>;

            fn into_static(self) -> Self::Output {
                #[allow(unused_imports)]
                use $crate::IntoStatic;

                $struct_name {
                    $($field: self.$field.into_static(),)+
                }
            }
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl $crate::IntoStatic for $enum_name {
            type Output = $enum_name;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                self
            }
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::IntoStatic for $enum_name<$lifetime> {
            type Output = $enum_name<'static>;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                match self {
                    $(
                        Self::$variant(value) => $enum_name::$variant(value.into_static()),
                    )+
                }
            }
        }
    };

    // owned enum (string-like)
    (enum $enum_name:ident string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl $crate::IntoStatic for $enum_name {
            type Output = $enum_name;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                self
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "core"))]
macro_rules! impl_into_static {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "core")]
macro_rules! impl_with_lifetime {
    // owned tuple struct (transparent)
    (struct $struct_name:ident transparent) => {
        #[automatically_derived]
        impl<'s> $crate::WithLifetime<'s> for $struct_name {
            type Lifetimed = $struct_name;
        }
    };

    // lifetimed tuple struct (transparent)
    (struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        #[automatically_derived]
        impl<'s> $crate::WithLifetime<'s> for $struct_name<$lifetime> {
            type Lifetimed = $struct_name<$lifetime>;
        }
    };

    // owned struct
    (struct $struct_name:ident { $($field:ident),* } $($rest:tt)*) => {
        #[automatically_derived]
        impl<'s> $crate::WithLifetime<'s> for $struct_name {
            type Lifetimed = $struct_name;
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* } $($rest:tt)*) => {
        #[automatically_derived]
        impl<$lifetime, 'instantiated_lifetime> $crate::WithLifetime<'instantiated_lifetime>
            for $struct_name<$lifetime>
        {
            type Lifetimed = $struct_name<'instantiated_lifetime>;
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<'s> $crate::WithLifetime<'s> for $enum_name {
            type Lifetimed = $enum_name;
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime, 'instantiated_lifetime> $crate::WithLifetime<'instantiated_lifetime>
            for $enum_name<$lifetime>
        {
            type Lifetimed = $enum_name<'instantiated_lifetime>;
        }
    };

    // owned enum (string-like)
    (enum $enum_name:ident string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<'s> $crate::WithLifetime<'s> for $enum_name {
            type Lifetimed = $enum_name;
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "core"))]
macro_rules! impl_with_lifetime {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "core")]
macro_rules! impl_serialize {
    // owned tuple struct (transparent)
    (struct $struct_name:ident transparent) => {
        #[automatically_derived]
        impl $crate::Serialize for $struct_name {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    self.0.serialize(serializer).await
                }
            }
        }
    };

    // lifetimed tuple struct (transparent)
    (struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        #[automatically_derived]
        impl<$lifetime> $crate::Serialize for $struct_name<$lifetime> {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    self.0.serialize(serializer).await
                }
            }
        }
    };

    // lifetimed struct
    (struct $struct_name:ident < $lifetime:lifetime > { $($field:ident),* }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::Serialize for $struct_name<$lifetime> {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    serializer
                        .write($crate::Event::MapStart($crate::MapStart {
                            size_hint: Some($crate::count_ident_tokens!($($field)*)),
                        }))
                        .await?;
                    $(
                        serializer.write($crate::Event::Str($crate::CowStr::Borrowed(stringify!($field)))).await?;
                        self.$field.serialize(serializer).await?;
                    )+
                    serializer.write($crate::Event::MapEnd).await
                }
            }
        }
    };

    // owned struct
    (struct $struct_name:ident { $($field:ident),* }) => {
        #[automatically_derived]
        impl $crate::Serialize for $struct_name {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    serializer
                        .write($crate::Event::MapStart($crate::MapStart {
                            size_hint: Some($crate::count_ident_tokens!($($field)*)),
                        }))
                        .await?;
                    $(
                        serializer.write($crate::Event::Str($crate::CowStr::Borrowed(stringify!($field)))).await?;
                        self.$field.serialize(serializer).await?;
                    )+
                    serializer.write($crate::Event::MapEnd).await
                }
            }
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl $crate::Serialize for $enum_name {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    serializer
                        .write($crate::Event::MapStart($crate::MapStart {
                            size_hint: Some(1),
                        }))
                        .await?;

                    match self {
                        $(
                            Self::$variant(value) => {
                                serializer.write($crate::Event::Str($crate::CowStr::Borrowed($variant_str))).await?;
                                value.serialize(serializer).await?;
                            }
                        )+
                    }

                    serializer.write($crate::Event::MapEnd).await
                }
            }
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::Serialize for $enum_name<$lifetime> {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    serializer
                        .write($crate::Event::MapStart($crate::MapStart {
                            size_hint: Some(1),
                        }))
                        .await?;

                    match self {
                        $(
                            Self::$variant(value) => {
                                serializer.write($crate::Event::Str($crate::CowStr::Borrowed($variant_str))).await?;
                                value.serialize(serializer).await?;
                            }
                        )+
                    }

                    serializer.write($crate::Event::MapEnd).await
                }
            }
        }
    };

    // owned enum (string-like)
    (enum $enum_name:ident string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl $crate::Serialize for $enum_name {
            #[allow(clippy::manual_async_fn)]
            fn serialize<'fut>(
                &'fut self,
                serializer: &'fut mut dyn $crate::DynSerializer,
            ) -> impl ::std::future::Future<Output = Result<(), $crate::MerdeError<'static>>> + 'fut {
                async move {
                    match self {
                        $(
                            Self::$variant => {
                                serializer.write($crate::Event::Str($crate::CowStr::Borrowed($variant_str))).await
                            }
                        )+
                    }
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "core"))]
macro_rules! impl_serialize {
    ($($rest:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_trait {
    // generic
    (Deserialize for $($rest:tt)*) => {
        $crate::impl_deserialize!($($rest)*);
        $crate::impl_into_static!($($rest)*);
        $crate::impl_with_lifetime!($($rest)*);
    };

    (Serialize for $($rest:tt)*) => {
        $crate::impl_serialize!($($rest)*);
    };
}

/// Derives the specified traits for a struct.
///
/// This macro can be used to generate implementations of [`Serialize`] and [`Deserialize`],
/// traits for a given struct.
///
/// # Usage
///
/// ```rust
/// struct MyStruct<'s> {
///     a: merde::CowStr<'s>,
///     b: i32,
///     c: bool,
/// }
///
/// merde::derive! {
///     impl (Serialize, Deserialize) for struct MyStruct<'s> { a, b, c }
/// }
/// ```
///
/// In this example, both traits are derived, of course you can omit the ones you don't need.
///
/// Structs without lifetime parameters are also allowed: the macro can tell the difference
/// from the fact that the lifetime parameter is not present in the invocation:
///
/// ```rust
/// struct MyStruct {
///     a: String,
///     b: i32,
///     c: bool,
/// }
///
/// merde::derive! {
///     //                          no lifetime param here 👇
///     impl (Serialize, Deserialize) for struct MyStruct { a, b, c }
/// }
/// ```
///
/// 1-tuple structs (newtypes) are supported, only in the "transparent" style: serializing
/// a `String` or a `MyStruct` will give `"foobar"` all the same, as if the newtype wrapper
/// was stripped, or, well, transparent:
///
/// ```rust
/// struct MyStruct(String);
///
/// merde::derive! {
///     impl (Serialize, Deserialize) for struct MyStruct transparent
/// }
///
/// assert_eq!(
///   merde::json::to_string(&MyStruct("foobar".into())).unwrap(),
///   r#""foobar""#
/// );
/// ```
///
/// Externally tagged enums are also supported. For both owned and lifetimed variants:
///
/// ```rust
/// enum MyEnum {
///     Variant1(String),
///     Variant2(i32),
/// }
///
/// merde::derive! {
///     impl (Serialize, Deserialize) for enum MyEnum
///     externally_tagged {
///         "variant1" => Variant1,
///         "variant2" => Variant2,
///     }
/// }
///
/// enum MyLifetimedEnum<'a> {
///     Variant1(merde::CowStr<'a>),
///     Variant2(i32),
/// }
///
/// merde::derive! {
///     impl (Serialize, Deserialize) for enum MyLifetimedEnum<'wot>
///     externally_tagged {
///         "variant1" => Variant1,
///         "variant2" => Variant2,
///     }
/// }
/// ```
///
/// This will serialize `MyEnum::Variant1("hello".into())` as `{"variant1":"hello"}`,
/// and `MyEnum::Variant2(42)` as `{"variant2":42}`.
#[macro_export]
macro_rules! derive {
    // generic
    (impl ($first_trait:ident, $($rest_traits:ident),*) for $($rest:tt)*) => {
        $crate::impl_trait!($first_trait for $($rest)*);
        $crate::derive!(impl ($($rest_traits),*) for $($rest)*);
    };
    (impl ($first_trait:ident) for $($rest:tt)*) => {
        $crate::impl_trait!($first_trait for $($rest)*);
    };
    (impl () for $($rest:tt)*) => {};
}

/// Returns an `Option<T>` from a closure that returns a `T` (which
/// is never called) — this is a type inference trick used when deserializing
/// struct fields
#[doc(hidden)]
pub fn none_of<I, T>(_f: impl FnOnce(I) -> T) -> Option<T> {
    None
}

#[doc(hidden)]
#[macro_export]
macro_rules! count_ident_tokens {
    () => { 0 };
    ($first:ident) => { 1 };
    ($first:ident $($rest:ident)*) => {
        1 + $crate::count_ident_tokens!($($rest)*)
    };
}
#[cfg(test)]
#[cfg(feature = "json")]
mod json_tests {
    use std::collections::HashMap;

    use super::*;
    use crate::json::from_str;

    #[test]
    fn test_complex_structs() {
        use std::borrow::Cow;
        use std::collections::HashMap;

        #[derive(Debug, PartialEq)]
        struct SecondStruct<'s> {
            string_field: Cow<'s, str>,
            int_field: i32,
        }

        derive! {
            impl (Serialize, Deserialize) for struct SecondStruct<'s> {
                string_field,
                int_field
            }
        }

        #[derive(Debug, PartialEq)]
        struct ComplexStruct<'s> {
            string_field: Cow<'s, str>,
            u8_field: u8,
            u16_field: u16,
            u32_field: u32,
            u64_field: u64,
            i8_field: i8,
            i16_field: i16,
            i32_field: i32,
            i64_field: i64,
            usize_field: usize,
            bool_field: bool,
            option_field: Option<i32>,
            vec_field: Vec<i32>,
            hashmap_field: HashMap<String, i32>,
            second_struct_field: SecondStruct<'s>,
        }

        derive! {
            impl (Serialize, Deserialize) for struct ComplexStruct<'s> {
                string_field,
                u8_field,
                u16_field,
                u32_field,
                u64_field,
                i8_field,
                i16_field,
                i32_field,
                i64_field,
                usize_field,
                bool_field,
                option_field,
                vec_field,
                hashmap_field,
                second_struct_field
            }
        }

        let mut hashmap = HashMap::new();
        hashmap.insert("key".to_string(), 42);

        let original = ComplexStruct {
            string_field: Cow::Borrowed("test string"),
            u8_field: 8,
            u16_field: 16,
            u32_field: 32,
            u64_field: 64,
            i8_field: -8,
            i16_field: -16,
            i32_field: -32,
            i64_field: -64,
            usize_field: 100,
            bool_field: true,
            option_field: Some(42),
            vec_field: vec![1, 2, 3],
            hashmap_field: hashmap,
            second_struct_field: SecondStruct {
                string_field: Cow::Borrowed("nested string"),
                int_field: 100,
            },
        };

        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: ComplexStruct = from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_u8_zero() {
        let original: u8 = 0;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: u8 = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_u8_max() {
        let original: u8 = u8::MAX;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: u8 = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_i8_min() {
        let original: i8 = i8::MIN;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: i8 = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_i8_max() {
        let original: i8 = i8::MAX;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: i8 = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_i64_min() {
        let original: i64 = i64::MIN;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: i64 = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_i64_max() {
        let original: i64 = i64::MAX;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: i64 = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_string_owned() {
        let original = String::from("Hello, World!");
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: String = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_string_borrowed() {
        let original: &str = "Hello, World!";
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: String = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_vec_empty() {
        let original: Vec<i32> = Vec::new();
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: Vec<i32> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_vec_non_empty() {
        let original = vec![1, 2, 3, 4, 5];
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: Vec<i32> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_hashmap_empty() {
        let original: HashMap<String, i32> = HashMap::new();
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: HashMap<String, i32> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_hashmap_non_empty() {
        let mut original = HashMap::new();
        original.insert("key1".to_string(), 42);
        original.insert("key2".to_string(), -10);
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: HashMap<String, i32> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_option_some() {
        let original: Option<i32> = Some(42);
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: Option<i32> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_option_none() {
        let original: Option<i32> = None;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: Option<i32> = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_bool_true() {
        let original = true;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: bool = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_bool_false() {
        let original = false;
        let serialized = crate::json::to_string(&original).unwrap();
        let deserialized: bool = from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}

// used to test out doc-tests
mod doctest_playground {

    #[allow(unused_imports)]
    use crate as merde;

    ////////////////////////////////////////////////////////////////////////////////

    // (insert doctest here for testing, with `#[test]` above fn main())

    ////////////////////////////////////////////////////////////////////////////////
}
