use std::borrow::Cow;

use merde_json::{
    Fantome, JsonArrayExt, JsonDeserialize, JsonSerialize, JsonSerializer, JsonValue, JsonValueExt,
};

#[derive(Debug, PartialEq)]
struct MixedArray<'src> {
    _boo: Fantome<'src>,

    items: Vec<JsonValue<'src>>,
}
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray { items }
}

#[derive(Debug, PartialEq)]
struct Items<'src> {
    _boo: Fantome<'src>,

    number: u32,
    string: Cow<'src, str>,
    boolean: bool,
}

impl<'src> JsonDeserialize<'src> for Items<'src> {
    fn json_deserialize<'val>(
        value: Option<&'val JsonValue<'src>>,
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
struct MixedArray2<'src> {
    _boo: Fantome<'src>,

    items: Items<'src>,
}
merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray2 { items }
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
