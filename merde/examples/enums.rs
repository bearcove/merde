use merde::CowStr;

fn main() {
    use merde::json::JsonSerialize;

    let events = vec![
        Event::MouseUp(MouseUp { x: 10, y: 20 }),
        Event::MouseDown(MouseDown { x: 30, y: 40 }),
        Event::TextInput(TextInput {
            text: "Hello".into(),
        }),
    ];

    for event in events {
        let json = event.to_json_string();
        println!("JSON: {}", json);
        let deserialized: Event = merde::json::from_str_via_value(&json).unwrap();
        println!("Deserialized: {:?}", deserialized);
        assert_eq!(event, deserialized);
    }

    println!("All events successfully round-tripped through JSON!");
}

#[derive(Debug, PartialEq, Eq)]
enum Event<'s> {
    MouseUp(MouseUp),
    MouseDown(MouseDown),
    TextInput(TextInput<'s>),
    StringStuff(StringStuff<'s>),
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for enum Event<'s>
    externally_tagged {
        "mouseup" => MouseUp,
        "mousedown" => MouseDown,
        "textinput" => TextInput,
        "stringstuff" => StringStuff,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseUp {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for struct MouseUp {
        x,
        y
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseDown {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for struct MouseDown {
        x,
        y
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TextInput<'s> {
    text: CowStr<'s>,
}

merde::derive! { impl (JsonSerialize, ValueDeserialize) for struct TextInput<'s> { text } }

#[derive(Debug, PartialEq, Eq)]
struct StringStuff<'s>(CowStr<'s>);

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for struct StringStuff<'s> transparent
}
