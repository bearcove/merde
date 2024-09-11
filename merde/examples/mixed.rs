use std::borrow::Cow;

use merde::MerdeError;
use merde_json::{JsonSerialize, JsonSerializer};
use merde_core::{Value, ValueDeserialize};

#[derive(Debug, PartialEq)]
struct MixedArray<'s> {
    items: Vec<Value<'s>>,
}
merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for MixedArray<'s> { items }
}

#[derive(Debug, PartialEq)]
struct Items<'s> {
    number: u32,
    string: Cow<'s, str>,
    boolean: bool,
}

impl<'s> ValueDeserialize<'s> for Items<'s> {
    fn from_value_ref<'val>(value: Option<&'val Value<'s>>) -> Result<Self, MerdeError> {
        let arr = value.ok_or(MerdeError::MissingValue)?.as_array()?;

        Ok(Items {
            number: arr.must_get(0)?,
            string: arr.must_get(1)?,
            boolean: arr.must_get(2)?,
        })
    }
}

impl JsonSerialize for Items<'_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer
            .write_arr()
            .elem(&self.number)
            .elem(&self.string)
            .elem(&self.boolean);
    }
}

#[derive(Debug, PartialEq)]
struct MixedArray2<'s> {
    items: Items<'s>,
}
merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for MixedArray2<'s> { items }
}

fn main() {
    let input = r#"
        {
            "items": [
                42, "foo", true
            ]
        }
    "#;

    let ma: MixedArray = merde_json::from_str_via_value(input).unwrap();
    println!("{:?}", ma);

    let ma: MixedArray2 = merde_json::from_str_via_value(input).unwrap();
    println!("{:?}", ma);
}
