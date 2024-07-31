use std::borrow::Cow;

use merde_json::{Fantome, JsonSerialize, ToRustValue};

fn main() {
    let input = r#"
    {
        "name": "John Doe",
        "age": 42,
        "address": {
            "street": "123 Main St",
            "city": "Anytown",
            "state": "CA",
            "zip": 12345
        }
    }
    "#;

    // Note: those two bindings are necessary — `Person` borrows from `JsonValue`
    // If we wanted a `Person` we can move, we could do `.to_static()`
    let person = merde_json::from_str(input).unwrap();
    let person: Person = person.to_rust_value().unwrap();
    println!("{:#?}", person);

    // Round-trip! Again, every binding borrows from the previous one, and
    // everything can be converted from `F<'a>` to `F<'static>` via the
    // `ToStatic` trait.
    let serialized = person.to_json_string();
    let person2 = merde_json::from_str(&serialized).unwrap();
    let person2: Person = person2.to_rust_value().unwrap();
    println!("{:#?}", person2);

    assert_eq!(person, person2);
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Address<'src, 'val> {
    _boo: Fantome<'src, 'val>,

    street: Cow<'val, str>,
    city: Cow<'val, str>,
    state: Cow<'val, str>,
    zip: u16,
}

merde_json::derive! {
    impl (JsonSerialize, JsonDeserialize) for Address {
        street,
        city,
        state,
        zip
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Person<'src, 'val> {
    _boo: Fantome<'src, 'val>,

    name: Cow<'val, str>,
    age: u8,
    address: Address<'src, 'val>,
}

merde_json::derive! {
    impl (JsonSerialize, JsonDeserialize) for Person { name, age, address }
}
