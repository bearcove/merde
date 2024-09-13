#[cfg(feature = "core")]
use merde::CowStr;

#[cfg(not(feature = "core"))]
type CowStr<'s> = std::borrow::Cow<'s, str>;

#[cfg(all(feature = "core", feature = "json"))]
fn main() {
    use merde::json::JsonSerialize;

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

#[cfg(not(all(feature = "core", feature = "json")))]
fn main() {
    eprintln!("Well if the `core` feature is not enabled,");
    eprintln!("we can't call `from_str_via_value` and stuff,");
    eprintln!("but this still serves as an example that");
    eprintln!("you can keep your `merde::derive!` in place,");
    eprintln!("they'll just not generate any code.");
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
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

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct Person<'s> {
    name: CowStr<'s>,
    age: u8,
    address: Address<'s>,
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for Person<'s> { name, age, address }
}
