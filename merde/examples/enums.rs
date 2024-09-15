use merde::CowStr;

fn main() {
    use merde::json::JsonSerialize;

    // TODO: fill out
}

#[derive(Debug, PartialEq, Eq)]
enum Event<'s> {
    MouseUp(MouseUp),
    MouseDown(MouseDown),
    TextInput(TextInput<'s>),
}

merde::derive! {
    impl (JsonSerialize, ValueDeserialize) for enum Event<'s>
    externally_tagged {
        "mouseup" => MouseUp,
        "mousedown" => MouseDown,
        "textinput" => TextInput,
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
