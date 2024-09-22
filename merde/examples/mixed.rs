use merde::CowStr;
use merde::Value;

#[derive(Debug, PartialEq)]
struct MixedArray<'s> {
    items: Vec<Value<'s>>,
}
merde::derive! {
    impl (JsonSerialize, Deserialize) for struct MixedArray<'s> { items }
}

#[derive(Debug, PartialEq)]
struct MixedArray2<'s> {
    items: (u64, CowStr<'s>, bool),
}
merde::derive! {
    impl (JsonSerialize, Deserialize) for struct MixedArray2<'s> { items }
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
