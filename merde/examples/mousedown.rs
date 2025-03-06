#[derive(Debug)]
enum TestEvent {
    MouseUp(MouseUp),
    MouseDown(MouseDown),
}

merde::derive! {
    impl (Serialize, Deserialize) for enum TestEvent
    externally_tagged {
        "mouseup" => MouseUp,
        "mousedown" => MouseDown,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseUp {
    x: i32,
    y: i32,
}

merde::derive! {
    impl (Serialize, Deserialize) for struct MouseUp {
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
    impl (Serialize, Deserialize) for struct MouseDown {
        x,
        y
    }
}

fn main() {
    let input = r#"{"mouseup": {"x": 100, "y": 200}}"#;
    let event: TestEvent = merde::json::from_str(input).unwrap();
    println!("TestEvent: {:?}", event);
}
