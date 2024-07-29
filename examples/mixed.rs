use std::marker::PhantomData;

use merde_json::{JsonValue, ToRustValue};

#[derive(Debug, PartialEq)]
struct MixedArray<'inner, 'borrow> {
    items: Vec<&'borrow JsonValue<'inner>>,
    _phantom: PhantomData<(&'inner (), &'borrow ())>,
}

merde_json::derive! {
    impl(JsonSerialize, JsonDeserialize) for MixedArray { items }
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
}
