#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "json")]
pub use merde_json as json;

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
macro_rules! impl_value_deserialize {
    // owned struct
    (struct $struct_name:ident { $($field:ident),+ }) => {
        #[automatically_derived]
        impl<'s> $crate::ValueDeserialize<'s> for $struct_name
        {
            fn from_value_ref(
                value: Option<&$crate::Value<'_>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let obj = value.ok_or(MerdeError::MissingValue)?.as_map()?;
                Ok($struct_name {
                    $($field: obj.must_get(stringify!($field))?,)+
                })
            }

            fn from_value(
                value: Option<$crate::Value<'_>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut obj = value.ok_or(MerdeError::MissingValue)?.into_map()?;
                Ok($struct_name {
                    $($field: obj.must_remove(stringify!($field))?,)+
                })
            }
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::ValueDeserialize<$lifetime> for $struct_name<$lifetime>
        {
            fn from_value_ref<'val>(
                value: Option<&'val $crate::Value<$lifetime>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let obj = value.ok_or(MerdeError::MissingValue)?.as_map()?;
                Ok($struct_name {
                    $($field: obj.must_get(stringify!($field))?,)+
                })
            }

            fn from_value(
                value: Option<$crate::Value<$lifetime>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut obj = value.ok_or(MerdeError::MissingValue)?.into_map()?;
                Ok($struct_name {
                    $($field: obj.must_remove(stringify!($field))?,)+
                })
            }
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        #[automatically_derived]
        impl $crate::ValueDeserialize for $enum_name {
            fn from_value_ref<'val>(
                value: Option<&'val $crate::Value<'_>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let obj = value.ok_or(MerdeError::MissingValue)?.as_map()?;
                Ok($enum_name::$variant(obj.must_get(stringify!($variant_str))?))
            }

            fn from_value(
                value: Option<$crate::Value<'_>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let mut obj = value.ok_or(MerdeError::MissingValue)?.into_map()?;
                Ok($enum_name::$variant(obj.must_remove(stringify!($variant_str))?))
            }
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::ValueDeserialize<$lifetime> for $enum_name<$lifetime> {
            fn from_value_ref<'val>(
                value: Option<&'val $crate::Value<$lifetime>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let map = value.ok_or(MerdeError::MissingValue)?.as_map()?;
                let (key, val) = map.iter().next().ok_or(MerdeError::MissingValue)?;
                match key.as_ref() {
                    $($variant_str => Ok($enum_name::$variant($crate::ValueDeserialize::from_value_ref(Some(val))?)),)*
                    _ => Err(MerdeError::UnknownProperty(key.to_string())),
                }
            }

            fn from_value(
                value: Option<$crate::Value<$lifetime>>,
            ) -> Result<Self, $crate::MerdeError> {
                #[allow(unused_imports)]
                use $crate::MerdeError;

                let map = value.ok_or(MerdeError::MissingValue)?.into_map()?;
                let (key, val) = map.into_iter().next().ok_or(MerdeError::MissingValue)?;
                match key.as_ref() {
                    $($variant_str => Ok($enum_name::$variant($crate::ValueDeserialize::from_value(Some(val))?)),)*
                    _ => Err(MerdeError::UnknownProperty(key.to_string())),
                }
            }
        }
    };
}

#[doc(hidden)]
#[cfg(not(feature = "deserialize"))]
#[macro_export]
macro_rules! impl_value_deserialize {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "core")]
macro_rules! impl_into_static {
    // owned struct
    (struct $struct_name:ident { $($field:ident),+ }) => {
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
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
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
        $($variant_str:literal => $variant:ident),+ $(,)?
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
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        #[automatically_derived]
        impl<$lifetime> $crate::IntoStatic for $enum_name<$lifetime> {
            type Output = $enum_name<$lifetime>;

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
    // owned struct
    (struct $struct_name:ident { $($field:ident),+ }) => {
        impl<'s> $crate::WithLifetime<'s> for $struct_name {
            type Lifetimed = $struct_name;
        }
    };

    // lifetimed struct
    (struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        impl<$lifetime, 'instantiated_lifetime> $crate::WithLifetime<'instantiated_lifetime>
            for $struct_name<$lifetime>
        {
            type Lifetimed = $struct_name<'instantiated_lifetime>;
        }
    };

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        impl<'s> $crate::WithLifetime<'s> for $enum_name {
            type Lifetimed = $enum_name;
        }
    };

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        impl<'s> $crate::WithLifetime<'s> for $enum_name<$lifetime> {
            type Lifetimed = $enum_name<$lifetime>;
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
    // lifetimed struct
    (struct $struct_name:ident < $lifetime:lifetime > { $($field:ident),+ }) => {
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
    (struct $struct_name:ident { $($field:ident),+ }) => {
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

    // lifetimed enum (externally tagged)
    (enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
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

    // owned enum (externally tagged)
    (enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
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
    // owned struct
    (@impl JsonSerialize, struct $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_json_serialize!(struct $struct_name { $($field),+ });
    };
    // lifetimed struct
    (@impl JsonSerialize, struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_json_serialize!(struct $struct_name <$lifetime> { $($field),+ });
    };
    // owned enum (externally tagged)
    (@impl JsonSerialize, enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        $crate::impl_json_serialize!(enum $enum_name externally_tagged {
            $($variant_str => $variant),+
        });
    };
    // lifetimed enum (externally tagged)
    (@impl JsonSerialize, enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        $crate::impl_json_serialize!(enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),+
        });
    };

    // owned struct
    (@impl ValueDeserialize, struct $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_value_deserialize!(struct $struct_name { $($field),+ });
        $crate::impl_into_static!(struct $struct_name { $($field),+ });
        $crate::impl_with_lifetime!(struct $struct_name { $($field),+ });
    };
    // lifetimed struct
    (@impl ValueDeserialize, struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_value_deserialize!(struct $struct_name <$lifetime> { $($field),+ });
        $crate::impl_into_static!(struct $struct_name <$lifetime> { $($field),+ });
        $crate::impl_with_lifetime!(struct $struct_name <$lifetime> { $($field),+ });
    };
    // owned enum (externally tagged)
    (@impl ValueDeserialize, enum $enum_name:ident externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        $crate::impl_value_deserialize!(enum $enum_name externally_tagged {
            $($variant_str => $variant),+
        });
    };
    // lifetimed enum (externally tagged)
    (@impl ValueDeserialize, enum $enum_name:ident <$lifetime:lifetime> externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        $crate::impl_value_deserialize!(enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),+
        });
    };
}

/// Derives the specified traits for a struct.
///
/// This macro can be used to generate implementations of [`JsonSerialize`], [`ValueDeserialize`],
/// and [`IntoStatic`] traits for a given struct.
///
/// # Usage
///
/// ```rust
/// use merde::ValueDeserialize;
/// use merde::CowStr;
/// use merde::json::JsonSerialize;
///
/// #[derive(Debug, PartialEq)]
/// struct MyStruct<'s> {
///     field1: CowStr<'s>,
///     field2: i32,
///     field3: bool,
/// }
///
/// merde::derive! {
///     impl (JsonSerialize, ValueDeserialize) for struct MyStruct<'s> {
///         field1,
///         field2,
///         field3
///     }
/// }
/// ```
///
/// In this example, all three traits are derived, of course you can omit the ones you don't need.
///
/// The struct must have exactly one lifetime parameter. Additionally, even if there are no
/// borrowed fields, the struct must include a `_phantom` field of type `PhantomData<&'a ()>`,
/// where `'a` is the lifetime parameter.
///
/// Implementing other variants (no lifetimes, multiple lifetimes, etc.) with declarative macros
/// would be too complicated. At this point we'd want a whole parser / compiler / code generator
/// for this — or a proc macro, see [serde](https://serde.rs/)'s serde_derive.
#[macro_export]
macro_rules! derive {
    // owned structs
    (impl ($($trait:ident),+) for struct $struct_name:ident { $($field:ident),+ }) => {
        $crate::derive!(@step1 { $($trait),+ } struct $struct_name { $($field),+ });
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
    (impl ($($trait:ident),+) for struct $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::derive!(@step1 { $($trait),+ } struct $struct_name <$lifetime> { $($field),+ });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } struct $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name <$lifetime> $fields);
        $crate::derive!(@step1 { $($rest_traits),* } struct $struct_name <$lifetime> $fields);
    };
    (@step1 { $trait:ident } struct $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, struct $struct_name <$lifetime> $fields);
    };
    (@step1 { } struct $struct_name:ident <$lifetime:lifetime> $fields:tt) => {};

    // owned enums
    (impl ($($trait:ident),+) for enum $enum_name:ident
    externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        $crate::derive!(@step1 { $($trait),+ } enum $enum_name externally_tagged {
            $($variant_str => $variant),+
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

    // lifetimed enums
    (impl ($($trait:ident),+) for enum $enum_name:ident <$lifetime:lifetime>
    externally_tagged {
        $($variant_str:literal => $variant:ident),+ $(,)?
    }) => {
        $crate::derive!(@step1 { $($trait),+ } enum $enum_name <$lifetime> externally_tagged {
            $($variant_str => $variant),+
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
}

#[cfg(test)]
#[cfg(feature = "json")]
mod json_tests {
    use super::*;
    use crate::json::{from_str_via_value, JsonSerialize};

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
            impl (JsonSerialize, ValueDeserialize) for struct SecondStruct<'s> {
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
            impl (JsonSerialize, ValueDeserialize) for struct ComplexStruct<'s> {
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
        let deserialized: ComplexStruct = from_str_via_value(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}

// used to test out doc-tests
mod doctest_playground {
    #[allow(unused_imports)]
    use crate as merde;

    ////////////////////////////////////////////////////////////////////////////////

    // (insert doctest here for testing)

    ////////////////////////////////////////////////////////////////////////////////
}
