use merde::{CowStr, DeserializeOwned};

fn deser_and_return<T>(s: String) -> Result<T, merde_json::MerdeError<'static>>
where
    T: DeserializeOwned,
{
    // here `s` is a `String`, but pretend we're making
    // a network request instead — the point is is that we
    // need to borrow from a local from the function body.
    let mut deser = merde_json::JsonDeserializer::new(&s);
    T::deserialize_owned(&mut deser).map_err(|e| e.to_static())
}

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

    let person: Person = merde_json::from_str(input).unwrap();
    println!("{:?}", person);

    let serialized = merde_json::to_string(&person).unwrap();
    let person2: Person = merde_json::from_str(&serialized).unwrap();
    println!("{:?}", person2);

    assert_eq!(person, person2);

    let person3 = deser_and_return::<Person>(serialized).unwrap();
    println!("{:?}", person3);

    assert_eq!(person, person3);
}

#[derive(Debug, PartialEq)]
struct Address<'s> {
    street: CowStr<'s>,
    city: CowStr<'s>,
    state: CowStr<'s>,
    zip: u16,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct Address<'s> {
        street,
        city,
        state,
        zip
    }
}

#[derive(Debug, PartialEq)]
struct Person<'s> {
    name: CowStr<'s>,
    age: u8,
    address: Address<'s>,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct Person<'s> { name, age, address }
}
