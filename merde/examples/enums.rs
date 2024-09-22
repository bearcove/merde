use merde::CowStr;

fn main() {
    use merde::json::JsonSerialize;

    let events = vec![
        Event::MouseUp(MouseUp { x: 10, y: 20 }),
        Event::MouseDown(MouseDown { x: 30, y: 40 }),
        Event::TextInput(TextInput {
            text: "Hello".into(),
        }),
        Event::StringStuff(StringStuff("Some string".into())),
        Event::Emergency(Emergency::NoPizzaLeft),
    ];

    for event in events {
        let json = event.to_json_string();
        println!("JSON: {}", json);
        let deserialized: Event = merde::json::from_str(&json).unwrap();
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
    Emergency(Emergency),
}

merde::derive! {
    impl (JsonSerialize, Deserialize) for enum Event<'s>
    externally_tagged {
        "mouseup" => MouseUp,
        "mousedown" => MouseDown,
        "textinput" => TextInput,
        "stringstuff" => StringStuff,
        "emergency" => Emergency,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseUp {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (JsonSerialize, Deserialize) for struct MouseUp {
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
    impl (JsonSerialize, Deserialize) for struct MouseDown {
        x,
        y
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TextInput<'s> {
    text: CowStr<'s>,
}

merde::derive! { impl (JsonSerialize, Deserialize) for struct TextInput<'s> { text } }

#[derive(Debug, PartialEq, Eq)]
struct StringStuff<'s>(CowStr<'s>);

merde::derive! {
    impl (JsonSerialize, Deserialize) for struct StringStuff<'s> transparent
}

#[derive(Debug, PartialEq, Eq)]
enum Emergency {
    NoPizzaLeft,
    CuddlesRequired,
    SmoothieReady,
}

merde::derive! {
    impl (JsonSerialize, Deserialize) for enum Emergency string_like {
        "nopizza" => NoPizzaLeft,
        "cuddles" => CuddlesRequired,
        "smoothie" => SmoothieReady,
    }
}
