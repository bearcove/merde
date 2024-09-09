use std::borrow::Cow;

use merde_json::{Fantome, ToStatic};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Address<'src> {
    _boo: Fantome<'src>,

    street: Cow<'src, str>,
    city: Cow<'src, str>,
    state: Cow<'src, str>,
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
struct Person<'src> {
    _boo: Fantome<'src>,

    name: Cow<'src, str>,
    age: u8,
    address: Address<'src>,
}

merde_json::derive! {
    impl (JsonDeserialize, ToStatic) for Person { name, age, address }
}

fn get_person() -> Person<'static> {
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

    let person: Person = merde_json::from_str(input).unwrap();
    person.to_static()
}

fn main() {
    let person = get_person();
    println!("{:?}", person);
}
