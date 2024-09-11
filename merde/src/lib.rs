#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "json")]
pub use merde_json as json;

#[cfg(feature = "time")]
pub use merde_time as time;

pub use merde_core::*;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_value_deserialize {
    ($struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
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

    ($struct_name:ident { $($field:ident),+ }) => {
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_into_static {
    ($struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
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

    ($struct_name:ident { $($field:ident),+ }) => {
        #[automatically_derived]
        impl $crate::ToStatic for $struct_name {
            type Output = $struct_name;

            #[inline(always)]
            fn into_static(self) -> Self::Output {
                self
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_serialize {
    ($struct_name:ident < $lifetime:lifetime > { $($field:ident),+ }) => {
        #[cfg(feature = "json")]
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

    ($struct_name:ident { $($field:ident),+ }) => {
        #[cfg(feature = "json")]
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
}

/// Derives the specified traits for a struct.
///
/// This macro can be used to automatically implement `JsonSerialize` and `ValueDeserialize`
/// traits for a given struct. It expands to call the appropriate implementation macros
/// based on the traits specified.
///
/// # Usage
///
/// ```rust
/// use merde::ValueDeserialize;
/// use merde::json::JsonSerialize;
/// use std::borrow::Cow;
///
/// #[derive(Debug, PartialEq)]
/// struct MyStruct<'s> {
///     field1: Cow<'s, str>,
///     field2: i32,
///     field3: bool,
/// }
///
/// merde::derive! {
///     impl(JsonSerialize, ValueDeserialize, ToStatic) for MyStruct<'s> {
///         field1,
///         field2,
///         field3
///     }
/// }
/// ```
///
/// This generates all three impls, but you can omit the ones you don't need.
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
    // cow variants
    (impl($($trait:ident),+) for $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::derive!(@step1 { $($trait),+ } $struct_name <$lifetime> { $($field),+ });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name <$lifetime> $fields);
        $crate::derive!(@step1 { $($rest_traits),* } $struct_name <$lifetime> $fields);
    };
    (@step1 { $trait:ident } $struct_name:ident <$lifetime:lifetime> $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name <$lifetime> $fields);
    };
    (@step1 { } $struct_name:ident <$lifetime:lifetime> $fields:tt) => {};

    // owned variants
    (impl($($trait:ident),+) for $struct_name:ident { $($field:ident),+ }) => {
        $crate::derive!(@step1 { $($trait),+ } $struct_name { $($field),+ });
    };
    (@step1 { $trait:ident, $($rest_traits:ident),* } $struct_name:ident $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name $fields);
        $crate::derive!(@step1 { $($rest_traits),* } $struct_name $fields);
    };
    (@step1 { $trait:ident } $struct_name:ident $fields:tt) => {
        $crate::impl_trait!(@impl $trait, $struct_name $fields);
    };
    (@step1 { } $struct_name:ident $fields:tt) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_trait {
    // borrowed
    (@impl JsonSerialize, $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_json_serialize!($struct_name <$lifetime> { $($field),+ });
    };
    // owned
    (@impl JsonSerialize, $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_json_serialize!($struct_name { $($field),+ });
    };

    // with lifetime param
    (@impl ValueDeserialize, $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_value_deserialize!($struct_name <$lifetime> { $($field),+ });
    };
    // l
    (@impl ValueDeserialize, $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_value_deserialize!($struct_name { $($field),+ });
    };

    // with lifetime param
    (@impl ToStatic, $struct_name:ident <$lifetime:lifetime> { $($field:ident),+ }) => {
        $crate::impl_into_static!($struct_name <$lifetime> { $($field),+ });
    };
    // without lifetime param
    (@impl ToStatic, $struct_name:ident { $($field:ident),+ }) => {
        $crate::impl_into_static!($struct_name { $($field),+ });
    };
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
            impl(JsonSerialize, ValueDeserialize) for SecondStruct<'s> {
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
            impl(JsonSerialize, ValueDeserialize) for ComplexStruct<'s> {
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
    // use crate as merde;
}
