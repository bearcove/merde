use merde::Value;

struct Person {
    first_name: String,
    last_name: String,
}

merde::derive! {
    impl (Deserialize) for struct Person { first_name, last_name }
}

fn main() {
    let input = "[".repeat(10_000);

    let value: Value<'static> = merde::json::from_str_owned(&input[..]).unwrap();
    eprintln!("value: {:?}", value);
}
