use std::borrow::Cow;
use merde_json::ToStatic;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Address<'s> {
    street: Cow<'s, str>,
    city: Cow<'s, str>,
    state: Cow<'s, str>,
    zip: u16,
}

merde_json::derive! {
    impl (JsonDeserialize, ToStatic) for Address<'s> {
        street,
        city,
        state,
        zip
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Person<'s> {
    name: Cow<'s, str>,
    age: u8,
    address: Address<'s>,
}

merde_json::derive! {
    impl (JsonDeserialize, ToStatic) for Person<'s> { name, age, address }
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