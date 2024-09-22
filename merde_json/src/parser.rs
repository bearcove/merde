use crate::{deserialize2::JsonDeserializer, MerdeJsonError};

use merde_core::{CowStr, Deserializer, Value};

pub(crate) fn json_str_to_value(src: &str) -> Result<Value<'_>, MerdeJsonError<'_>> {
    JsonDeserializer::new(src).deserialize()
}

pub(crate) fn cowify<'j>(src: &'j [u8], s: &str) -> CowStr<'j> {
    if src.as_ptr_range().contains(&s.as_ptr()) {
        CowStr::Borrowed(unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(s.as_ptr(), s.len()))
        })
    } else {
        CowStr::Owned(s.into())
    }
}

#[cfg(test)]
mod tests {
    use merde_core::{Array, CowStr, Map, Value};

    use crate::parser::{cowify, json_str_to_value};

    #[test]
    fn test_cowify() {
        let src = "That's a subset!";
        let s = &src[4..8];
        assert_eq!(cowify(src.as_bytes(), s), CowStr::Borrowed(s));

        let src = "Not a subset";
        let s = "indeed not";
        assert_eq!(cowify(src.as_bytes(), s), CowStr::Owned(s.into()));
    }

    #[test]
    fn test_jiter_to_value() {
        let src = r#"
        {
            "name": "John Doe",
            "age": 42,
            "address": {
                "street": "123 Main St",
                "city": "Anytown",
                "state": "CA",
                "zip": 12345
            },
            "friends": [
                "Alice",
                "Bob",
                "Charlie"
            ]
        }
        "#;

        let value = json_str_to_value(src).unwrap();
        assert_eq!(
            value,
            Value::Map(
                Map::new()
                    .with("name", Value::Str(CowStr::from("John Doe")))
                    .with("age", Value::Int(42))
                    .with(
                        "address",
                        Map::new()
                            .with("street", Value::Str(CowStr::from("123 Main St")))
                            .with("city", Value::Str(CowStr::from("Anytown")))
                            .with("state", Value::Str(CowStr::from("CA")))
                            .with("zip", Value::Int(12345))
                    )
                    .with(
                        "friends",
                        Array::new()
                            .with(Value::from("Alice"))
                            .with(Value::from("Bob"))
                            .with(Value::from("Charlie"))
                    )
            )
        );
    }
}
