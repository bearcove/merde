use std::borrow::Cow;

use merde_json::{
    JsonArrayExt, JsonSerialize, JsonSerializer, JsonValue, JsonValueExt, ValueDeserialize,
};

#[derive(Debug, PartialEq)]
struct MixedArray<'s> {
    items: Vec<JsonValue<'s>>,
}
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray<'s> { items }
}

#[derive(Debug, PartialEq)]
struct Items<'s> {
    number: u32,
    string: Cow<'s, str>,
    boolean: bool,
}

impl<'s> ValueDeserialize<'s> for Items<'s> {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'s>>,
    ) -> Result<Self, merde_json::MerdeJsonError> {
        let arr = value
            .ok_or(merde_json::MerdeJsonError::MissingValue)?
            .as_array()?;

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
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray2<'s> { items }
}

fn main() {
    let input = r#"
        {
            "items": [
                42, "foo", true
            ]
        }
    "#;

    let ma: MixedArray = merde_json::from_str(input).unwrap();
    println!("{:?}", ma);

    let ma: MixedArray2 = merde_json::from_str(input).unwrap();
    println!("{:?}", ma);
}
