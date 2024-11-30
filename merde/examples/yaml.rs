use std::collections::HashMap;

use merde::{DynDeserializerExt, Value};
use merde_core::Deserializer;
use merde_yaml::YamlDeserializer;

#[derive(Debug, PartialEq)]
struct ComplexStruct {
    name: String,
    age: u32,
    hobbies: Vec<String>,
    address: Address,
    scores: Vec<Score>,
}

merde::derive! {
    impl (Deserialize) for struct ComplexStruct {
        name,
        age,
        hobbies,
        address,
        scores
    }
}

#[derive(Debug, PartialEq)]
struct Address {
    street: String,
    city: String,
    country: String,
}

merde::derive! {
    impl (Deserialize) for struct Address {
        street,
        city,
        country
    }
}

#[derive(Debug, PartialEq)]
struct Score {
    subject: String,
    value: f32,
}

merde::derive! {
    impl (Deserialize) for struct Score {
        subject,
        value
    }
}

fn main() {
    let yaml = r#"
            name: John Doe
            age: 30
            hobbies:
              - reading
              - swimming
              - coding
            address:
              street: 123 Main St
              city: Anytown
              country: Wonderland
            scores:
              - subject: Math
                value: 95.5
              - subject: Science
                value: 88.0
              - subject: Literature
                value: 92.5
        "#;

    let mut de = YamlDeserializer::new(yaml);
    let result: ComplexStruct = de.deserialize_sync().unwrap();

    println!("Deserialized ComplexStruct: {result:#?}");

    let yaml_map = r#"
        1: 100
        "two": 200.5
        true: "three hundred"
        [1, 2, 3]: { "nested": "value" }
    "#;

    let mut de = YamlDeserializer::new(yaml_map);
    let result: HashMap<Value, Value> = de.deserialize_sync().unwrap();

    println!("Deserialized HashMap: {result:#?}");
}
