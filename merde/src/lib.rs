#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "json")]
pub use merde_json as json;

#[cfg(feature = "yaml")]
pub use merde_yaml as yaml;

#[cfg(feature = "json")]
#[allow(unused_imports)]
use json::JsonSerialize;

#[cfg(feature = "time")]
pub use merde_time as time;

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
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
            where
                D: $crate::Deserializer<'s> + ?Sized {
                Ok(Self(de.t().await?))
            }
        }
    };

    // lifetimed tuple struct (transparent)
    (struct $struct_name:ident <$s:lifetime> transparent) => {
        #[automatically_derived]
        impl<$s> $crate::Deserialize<$s> for $struct_name<$s> {
            #[inline(always)]
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<$s>>
            where
                D: $crate::Deserializer<'s> + ?Sized {
                Ok(Self(de.t().await?))
            }
        }
    };

    // owned struct
    (struct $struct_name:ident { $($field:ident),* }) => {
        #[automatically_derived]
        impl<'s> $crate::Deserialize<'s> for $struct_name {
            #[inline(always)]
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
            where
                D: $crate::Deserializer<'s> + ?Sized {

                #![allow(unreachable_code)]

                de.next()?.into_map_start()?;

                $(
                    let mut $field = $crate::none_of(|i: $struct_name| i.$field);
                )+

                loop {
                    match de.next()? {
                        $crate::Event::MapEnd => break,
                        $crate::Event::Str(key) => match key.as_ref() {
                            $(stringify!($field) => {
                                $field = Some(de.t().await?);
                            })*
                            _ => {
                                return Err($crate::MerdeError::UnknownProperty(key).into());
                            }
                        }
                        ev => {
                            return Err($crate::MerdeError::UnexpectedEvent {
                                got: $crate::EventType::from(&ev),
                                expected: &[$crate::EventType::Str, $crate::EventType::MapEnd],
                            }
                            .into())
                        }
                    }
                }

                Ok($struct_name {
                    $($field: $crate::Deserialize::from_option($field, stringify!($field).into())?,)+
                })
            }
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$s:lifetime> { $($field:ident),* }) => {
        #[automatically_derived]
        impl<$s> $crate::Deserialize<$s> for $struct_name<$s> {
            #[inline(always)]
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<$s>>
            where
                D: $crate::Deserializer<$s> + ?Sized {

                #![allow(unreachable_code)]

                de.next()?.into_map_start()?;

                $(
                    let mut $field = $crate::none_of(|i: $struct_name<$s>| i.$field);
                )+

                loop {
                    match de.next()? {
                        $crate::Event::MapEnd => break,
                        $crate::Event::Str(key) => match key.as_ref() {
                            $(stringify!($field) => {
                                $field = Some(de.t().await?);
                            })*
                            _ => {
                                return Err($crate::MerdeError::UnknownProperty(key).into());
                            }
                        }
                        ev => {
                            return Err($crate::MerdeError::UnexpectedEvent {
                                got: $crate::EventType::from(&ev),
                                expected: &[$crate::EventType::Str, $crate::EventType::MapEnd],
                            }
                            .into())
                        }
                    }
                }

                Ok($struct_name {
                    $($field: $crate::Deserialize::from_option($field, stringify!($field).into())?,)+
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
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
            where
                D: $crate::Deserializer<'s> + ?Sized,
            {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                de.next()?.into_map_start()?;
                let key = de.next()?.into_str()?;
                match key.as_ref() {
                    $($variant_str => {
                        let value = de.t().await?;
                        de.next()?.into_map_end()?;
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
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<$lifetime>>
            where
                D: $crate::Deserializer<$lifetime> + ?Sized,
            {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                de.next()?.into_map_start()?;
                let key = de.next()?.into_str()?;
                match key.as_ref() {
                    $($variant_str => {
                        let value = de.t().await?;
                        de.next()?.into_map_end()?;
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
            async fn deserialize<D>(de: &mut D) -> Result<Self, D::Error<'s>>
            where
                D: $crate::Deserializer<'s> + ?Sized,
            {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let s = de.next()?.into_str()?;
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
    (struct $struct_name:ident { $($field:ident),* }) => {
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
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* }) => {
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
    (struct $struct_name:ident { $($field:ident),* }) => {
        #[automatically_derived]
        impl<'s> $crate::WithLifetime<'s> for $struct_name {
            type Lifetimed = $struct_name;
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* }) => {
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
#[cfg(all(feature = "core", feature = "json"))]
macro_rules! impl_json_serialize {
    // owned tuple struct (transparent)
    (struct $struct_name:ident transparent) => {
        #[automatically_derived]
        impl $crate::json::JsonSerialize for $struct_name {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                self.0.json_serialize(serializer)
            }
        }
    };

    // lifetimed tuple struct (transparent)
    (struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        #[automatically_derived]
        impl<$lifetime> $crate::json::JsonSerialize for $struct_name<$lifetime> {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                self.0.json_serialize(serializer)
            }
        }
    };

    // lifetimed struct
    (struct $struct_name:ident < $lifetime:lifetime > { $($field:ident),* }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::json::JsonSerialize for $struct_name<$lifetime> {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut guard = serializer.write_obj();
                $(
                    guard.pair(stringify!($field), &self.$field);
                )+
            }
        }
    };

    // owned struct
    (struct $struct_name:ident { $($field:ident),* }) => {
        #[automatically_derived]
        impl $crate::json::JsonSerialize for $struct_name {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut guard = serializer.write_obj();
                $(
                    guard.pair(stringify!($field), &self.$field);
                )+
            }
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl $crate::json::JsonSerialize for $enum_name {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut guard = serializer.write_obj();
                match self {
                    $(
                        Self::$variant(value) => {
                            guard.pair($variant_str, &value);
                        }
                    )+
                }
            }
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::json::JsonSerialize for $enum_name<$lifetime> {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut guard = serializer.write_obj();
                match self {
                    $(
                        Self::$variant(value) => {
                            guard.pair($variant_str, &value);
                        }
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
        impl $crate::json::JsonSerialize for $enum_name {
            fn json_serialize(&self, serializer: &mut $crate::json::JsonSerializer) {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                match self {
                    $(
                        Self::$variant => {
                            serializer.write_str($variant_str);
                        }
                    )+
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(all(feature = "core", feature = "json")))]
macro_rules! impl_json_serialize {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_trait {
    //------------------------------------------------------------------------------------

    // owned struct
    (@impl JsonSerialize, struct $struct_name:ident { $($field:ident),* }) => {
        $crate::impl_json_serialize!(struct $struct_name { $($field),* });
    };
    // lifetimed struct
    (@impl JsonSerialize, struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* }) => {
        $crate::impl_json_serialize!(struct $struct_name <$lifetime> { $($field),* });
    };
    // owned enum (externally tagged)
    (@impl JsonSerialize, enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::impl_json_serialize!(enum $enum_name externally_tagged {
            $($variant_str => $variant),*
        });
    };
    // lifetimed enum (externally tagged)
    (@impl JsonSerialize, enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::impl_json_serialize!(enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),*
        });
    };
    // owned enum (string-like)
    (@impl JsonSerialize, enum $enum_name:ident string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::impl_json_serialize!(enum $enum_name string_like {
            $($variant_str => $variant),*
        });
    };
    // owned tuple struct (transparent)
    (@impl JsonSerialize, struct $struct_name:ident transparent) => {
        $crate::impl_json_serialize!(struct $struct_name transparent);
    };
    // lifetimed tuple struct (transparent)
    (@impl JsonSerialize, struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        $crate::impl_json_serialize!(struct $struct_name <$lifetime> transparent);
    };

    //------------------------------------------------------------------------------------

    // owned struct
    (@impl Deserialize, struct $struct_name:ident { $($field:ident),* }) => {
        $crate::impl_deserialize!(struct $struct_name { $($field),* });
        $crate::impl_into_static!(struct $struct_name { $($field),* });
        $crate::impl_with_lifetime!(struct $struct_name { $($field),* });
    };
    // lifetimed struct
    (@impl Deserialize, struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* }) => {
        $crate::impl_deserialize!(struct $struct_name <$lifetime> { $($field),* });
        $crate::impl_into_static!(struct $struct_name <$lifetime> { $($field),* });
        $crate::impl_with_lifetime!(struct $struct_name <$lifetime> { $($field),* });
    };
    // owned enum (externally tagged)
    (@impl Deserialize, enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::impl_deserialize!(enum $enum_name externally_tagged {
            $($variant_str => $variant),*
        });
        $crate::impl_into_static!(enum $enum_name externally_tagged {
            $($variant_str => $variant),*
        });
        $crate::impl_with_lifetime!(enum $enum_name externally_tagged {
            $($variant_str => $variant),*
        });
    };
    // lifetimed enum (externally tagged)
    (@impl Deserialize, enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::impl_deserialize!(enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),*
        });
        $crate::impl_into_static!(enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),*
        });
        $crate::impl_with_lifetime!(enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),*
        });
    };
    // owned enum (string-like)
    (@impl Deserialize, enum $enum_name:ident string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::impl_deserialize!(enum $enum_name string_like {
            $($variant_str => $variant),*
        });
        $crate::impl_into_static!(enum $enum_name string_like {
            $($variant_str => $variant),*
        });
        $crate::impl_with_lifetime!(enum $enum_name string_like {
            $($variant_str => $variant),*
        });
    };
    // owned tuple struct (transparent)
    (@impl Deserialize, struct $struct_name:ident transparent) => {
        $crate::impl_deserialize!(struct $struct_name transparent);
        $crate::impl_into_static!(struct $struct_name transparent);
        $crate::impl_with_lifetime!(struct $struct_name transparent);
    };
    // lifetimed tuple struct (transparent)
    (@impl Deserialize, struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        $crate::impl_deserialize!(struct $struct_name <$lifetime> transparent);
        $crate::impl_into_static!(struct $struct_name <$lifetime> transparent);
        $crate::impl_with_lifetime!(struct $struct_name <$lifetime> transparent);
    };
}

/// Derives the specified traits for a struct.
///
/// This macro can be used to generate implementations of [`JsonSerialize`] and [`Deserialize`],
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
///     impl (JsonSerialize, Deserialize) for struct MyStruct<'s> { a, b, c }
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
///     impl (JsonSerialize, Deserialize) for struct MyStruct { a, b, c }
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
///     impl (JsonSerialize, Deserialize) for struct MyStruct transparent
/// }
///
/// use merde::json::JsonSerialize;
/// assert_eq!(
///   MyStruct("foobar".into()).to_json_string(),
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
///     impl (JsonSerialize, Deserialize) for enum MyEnum
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
///     impl (JsonSerialize, Deserialize) for enum MyLifetimedEnum<'wot>
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
    // owned tuple structs (transparent)
    (impl ($($trait:ident),*) for struct $struct_name:ident transparent) => {
        $crate::derive!(@step1 { $($trait),* } struct $struct_name transparent);
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } struct $struct_name:ident transparent) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name transparent);
        $crate::derive!(@step1 { $($rest_traits),* } struct $struct_name transparent);
    };
    (@step1 { $trait:ident } struct $struct_name:ident transparent) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name transparent);
    };
    (@step1 { } struct $struct_name:ident transparent) => {};

    // lifetimed tuple structs (transparent)
    (impl ($($trait:ident),*) for struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        $crate::derive!(@step1 { $($trait),* } struct $struct_name <$lifetime> transparent);
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name <$lifetime> transparent);
        $crate::derive!(@step1 { $($rest_traits),* } struct $struct_name <$lifetime> transparent);
    };
    (@step1 { $trait:ident } struct $struct_name:ident <$lifetime:lifetime> transparent) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name <$lifetime> transparent);
    };
    (@step1 { } struct $enum_name:ident <$lifetime:lifetime> transparent) => {};

    // owned structs
    (impl ($($trait:ident),*) for struct $struct_name:ident { $($field:ident),* }) => {
        $crate::derive!(@step1 { $($trait),* } struct $struct_name { $($field),* });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } struct $struct_name:ident $fields:tt) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name $fields);
        $crate::derive!(@step1 { $($rest_traits),* } struct $struct_name $fields);
    };
    (@step1 { $trait:ident } struct $struct_name:ident $fields:tt) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name $fields);
    };
    (@step1 { } struct $struct_name:ident $fields:tt) => {};

    // lifetimed structs
    (impl ($($trait:ident),*) for struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),* }) => {
        $crate::derive!(@step1 { $($trait),* } struct $struct_name <$lifetime> { $($field),* });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } struct $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name <$lifetime> $fields);
        $crate::derive!(@step1 { $($rest_traits),* } struct $struct_name <$lifetime> $fields);
    };
    (@step1 { $trait:ident } struct $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name <$lifetime> $fields);
    };
    (@step1 { } struct $struct_name:ident <$lifetime:lifetime> $fields:tt) => {};

    // owned enums (externally tagged)
    (impl ($($trait:ident),*) for enum $enum_name:ident
    externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::derive!(@step1 { $($trait),* } enum $enum_name externally_tagged {
            $($variant_str => $variant),*
        });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } enum $enum_name:ident
    externally_tagged $variants:tt) => {
        $crate::impl_trait!(@impl $trait, enum $enum_name externally_tagged $variants);
        $crate::derive!(@step1 { $($rest_traits),* } enum $enum_name externally_tagged $variants);
    };
    (@step1 { $trait:ident } enum $enum_name:ident externally_tagged $variants:tt) => {
        $crate::impl_trait!(@impl $trait, enum $enum_name externally_tagged $variants);
    };
    (@step1 { } enum $enum_name:ident externally_tagged $variants:tt) => {};

    // lifetimed enums (externally tagged)
    (impl ($($trait:ident),*) for enum $enum_name:ident <$lifetime:lifetime>
    externally_tagged {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::derive!(@step1 { $($trait),* } enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),*
        });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } enum $enum_name:ident <$lifetime:lifetime>
    externally_tagged $variants:tt) => {
        $crate::impl_trait!(@impl $trait, enum $enum_name <$lifetime> externally_tagged $variants);
        $crate::derive!(@step1 { $($rest_traits),* } enum $enum_name <$lifetime> externally_tagged $variants);
    };
    (@step1 { $trait:ident } enum $enum_name:ident <$lifetime:lifetime> externally_tagged $variants:tt) => {
        $crate::impl_trait!(@impl $trait, enum $enum_name <$lifetime> externally_tagged $variants);
    };
    (@step1 { } enum $enum_name:ident <$lifetime:lifetime> externally_tagged $variants:tt) => {};

    // owned enums (string-like)
    (impl ($($trait:ident),*) for enum $enum_name:ident
    string_like {
        $($variant_str:literal => $variant:ident),* $(,)?
    }) => {
        $crate::derive!(@step1 { $($trait),* } enum $enum_name string_like {
            $($variant_str => $variant),*
        });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } enum $enum_name:ident
    string_like $variants:tt) => {
        $crate::impl_trait!(@impl $trait, enum $enum_name string_like $variants);
        $crate::derive!(@step1 { $($rest_traits),* } enum $enum_name string_like $variants);
    };
    (@step1 { $trait:ident } enum $enum_name:ident string_like $variants:tt) => {
        $crate::impl_trait!(@impl $trait, enum $enum_name string_like $variants);
    };
    (@step1 { } enum $enum_name:ident string_like $variants:tt) => {};
}

/// Returns an `Option<T>` from a closure that returns a `T` (which
/// is never called) — this is a type inference trick used when deserializing
/// struct fields
#[doc(hidden)]
pub fn none_of<I, T>(_f: impl FnOnce(I) -> T) -> Option<T> {
    None
}

#[cfg(test)]
#[cfg(feature = "json")]
mod json_tests {
    use super::*;
    use crate::json::{from_str, JsonSerialize};

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
            impl (JsonSerialize, Deserialize) for struct SecondStruct<'s> {
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
            impl (JsonSerialize, Deserialize) for struct ComplexStruct<'s> {
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

        let serialized = original.to_json_string();
        let deserialized: ComplexStruct = from_str(&serialized).unwrap();

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
