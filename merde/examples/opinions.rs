use merde::{CowStr, DeserOpinions, FieldSlot};

fn main() {
    let input_precise = r#"
        { "foo_bar": "hello" }
    "#;
    let o: Owned = merde_json::from_str(input_precise).unwrap();
    assert_eq!(o.foo_bar, "hello");
    eprintln!("{o:#?}");

    let input_camel_case = r#"
        { "fooBar": "hello" }
    "#;
    let o: Owned = merde_json::from_str(input_camel_case).unwrap();
    assert_eq!(o.foo_bar, "hello");
    eprintln!("{o:#?}");

    let input_too_many_fields = r#"
        { "foo_bar": "hello", "foo_bar2": {"cruel": "world"} }
    "#;
    assert!(merde_json::from_str::<Owned>(input_too_many_fields).is_err());
    let o: OwnedRelaxed = merde_json::from_str(input_too_many_fields).unwrap();
    assert_eq!(o.foo_bar, "hello");
    eprintln!("{o:#?}");

    let input_missing_field = r#"
        {}
    "#;
    let o: Owned = merde_json::from_str(input_missing_field).unwrap();
    assert_eq!(o.foo_bar, "(default)");
    eprintln!("{o:#?}");
}

#[derive(Debug)]
struct Owned {
    foo_bar: String,
}

struct OwnedOpinions;

impl DeserOpinions for OwnedOpinions {
    fn deny_unknown_fields(&self) -> bool {
        true
    }

    #[allow(clippy::needless_lifetimes)]
    fn default_field_value<'s, 'borrow>(&self, key: &'borrow str, slot: FieldSlot<'s, 'borrow>) {
        if key == "foo_bar" {
            slot.fill::<String>("(default)".into());
        }
    }

    fn map_key_name<'s>(&self, key: CowStr<'s>) -> CowStr<'s> {
        if key == "fooBar" {
            CowStr::Owned("foo_bar".into())
        } else {
            key
        }
    }
}

merde::derive! {
    impl (Deserialize) for struct Owned {
        foo_bar
    } via OwnedOpinions
}

#[derive(Debug)]
struct OwnedRelaxed {
    foo_bar: String,
}

merde::derive! {
    impl (Deserialize) for struct OwnedRelaxed {
        foo_bar
    }
}
