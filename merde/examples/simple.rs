use merde::CowStr;
use merde::json::JsonSerialize;

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
    let person: Person = merde_json::from_str_via_value(input).unwrap();
    println!("{:#?}", person);

    // Round-trip! Again, every binding borrows from the previous one, and
    // everything can be converted from `F<'a>` to `F<'static>` via the
    // `IntoStatic` trait.
    let serialized = person.to_json_string();
    let person2: Person = merde_json::from_str_via_value(&serialized).unwrap();
    println!("{:#?}", person2);

    assert_eq!(person, person2);
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Address<'s> {
    street: CowStr<'s>,
    city: CowStr<'s>,
    state: CowStr<'s>,
    zip: u16,
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for Address<'s> {
        street,
        city,
        state,
        zip
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct Person<'s> {
    name: CowStr<'s>,
    age: u8,
    address: Address<'s>,
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for Person<'s> { name, age, address }
}
