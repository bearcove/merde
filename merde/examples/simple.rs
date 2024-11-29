use merde::CowStr;

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
    let person: Person = merde_json::from_str(input).unwrap();
    println!("{:#?}", person);

    // Round-trip! Again, every binding borrows from the previous one, and
    // everything can be converted from `F<'a>` to `F<'static>` via the
    // `IntoStatic` trait.
    let serialized = merde_json::to_string(&person).unwrap();
    let person2: Person = merde_json::from_str(&serialized).unwrap();
    println!("{:#?}", person2);

    assert_eq!(person, person2);
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct Address<'s> {
    street: CowStr<'s>,
    city: CowStr<'s>,
    state: CowStr<'s>,
    zip: u16,
    extra: Option<CowStr<'s>>,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct Address<'s> {
        street,
        city,
        state,
        zip,
        extra
    }
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct Person<'s> {
    name: CowStr<'s>,
    age: u8,
    address: Address<'s>,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct Person<'s> { name, age, address }
}
