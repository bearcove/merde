use std::borrow::Cow;

use merde_json::{Fantome, ToRustValue, ToStatic};

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
    impl (JsonDeserialize, ToStatic) for Address {
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
    impl (JsonDeserialize, ToStatic) for Person { name, age, address }
}

fn get_person() -> Person<'static, 'static> {
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

    let person = merde_json::from_str(input).unwrap();
    let person: Person = person.to_rust_value().unwrap();
    person.to_static()
}

fn main() {
    let person = get_person();
    println!("{:?}", person);
}
