#[cfg(feature = "ahash")]
fn main() {
    use ahash::HashMap;
    use merde::json;

    let mut h = HashMap::default();
    h.insert("street".to_string(), "123 Main St".to_string());
    h.insert("city".to_string(), "Anytown".to_string());
    h.insert("state".to_string(), "CA".to_string());
    h.insert("zip".to_string(), "12345".to_string());

    println!("h: {:#?}", h);

    let serialized = json::to_string(&h);
    println!("serialized: {}", serialized);

    let deserialized: HashMap<String, String> = json::from_str(&serialized).unwrap();
    println!("deserialized: {:#?}", deserialized);

    assert_eq!(h, deserialized);

    assert_eq!(deserialized.get("street"), Some(&"123 Main St".to_string()));
    assert_eq!(deserialized.get("city"), Some(&"Anytown".to_string()));
    assert_eq!(deserialized.get("state"), Some(&"CA".to_string()));
    assert_eq!(deserialized.get("zip"), Some(&"12345".to_string()));
}
