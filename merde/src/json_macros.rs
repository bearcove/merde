#[doc(hidden)]
#[macro_export]
macro_rules! impl_json_serialize {
    ($struct_name:ident < $lifetime:lifetime > { $($field:ident),+ }) => {
        #[cfg(feature = "json")]
        #[automatically_derived]
        impl<$lifetime> $crate::merde_json::JsonSerialize for $struct_name<$lifetime> {
            fn json_serialize(&self, serializer: &mut $crate::merde_json::JsonSerializer) {
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
        impl $crate::merde_json::JsonSerialize for $struct_name {
            fn json_serialize(&self, serializer: &mut $crate::merde_json::JsonSerializer) {
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
