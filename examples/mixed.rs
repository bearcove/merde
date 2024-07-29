use std::borrow::Cow;

use jiter::JsonArray;
use merde_json::{
    Fantome, JsonArrayExt, JsonDeserialize, JsonSerialize, JsonSerializer, JsonValue, JsonValueExt,
    ToRustValue,
};

#[derive(Debug, PartialEq)]
struct MixedArray<'inner, 'borrow> {
    _boo: Fantome<'inner, 'borrow>,

    items: Vec<&'borrow JsonValue<'inner>>,
}
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray { items }
}

#[derive(Debug, PartialEq)]
struct MixedArray2<'inner, 'borrow> {
    _boo: Fantome<'inner, 'borrow>,

    items: &'borrow JsonArray<'inner>,
}
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray2 { items }
}

#[derive(Debug, PartialEq)]
struct Items<'inner, 'borrow> {
    _boo: Fantome<'inner, 'borrow>,

    number: u32,
    string: Cow<'borrow, str>,
    boolean: bool,
}

impl<'inner, 'borrow> JsonDeserialize<'inner, 'borrow> for Items<'inner, 'borrow>
where
    'inner: 'borrow,
{
    fn json_deserialize(
        value: Option<&'borrow JsonValue<'inner>>,
    ) -> Result<Self, merde_json::MerdeJsonError> {
        let arr = value
            .ok_or(merde_json::MerdeJsonError::MissingValue)?
            .as_array()?;

        Ok(Items {
            _boo: Default::default(),

            number: arr.must_get(0)?,
            string: arr.must_get(1)?,
            boolean: arr.must_get(2)?,
        })
    }
}

impl JsonSerialize for Items<'_, '_> {
    fn json_serialize(&self, serializer: &mut JsonSerializer) {
        serializer
            .write_arr()
            .elem(&self.number)
            .elem(&self.string)
            .elem(&self.boolean);
    }
}

#[derive(Debug, PartialEq)]
struct MixedArray3<'inner, 'borrow> {
    _boo: Fantome<'inner, 'borrow>,

    items: Items<'inner, 'borrow>,
}
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray3 { items }
}

fn main() {
    let input = r#"
        {
            "items": [
                42, "foo", true
            ]
        }
    "#;

    let ma = merde_json::from_str(input).unwrap();
    let ma: MixedArray = ma.to_rust_value().unwrap();
    println!("{:?}", ma);

    let ma = merde_json::from_str(input).unwrap();
    let ma: MixedArray2 = ma.to_rust_value().unwrap();
    println!("{:?}", ma);

    let ma = merde_json::from_str(input).unwrap();
    let ma: MixedArray3 = ma.to_rust_value().unwrap();
    println!("{:?}", ma);
}
