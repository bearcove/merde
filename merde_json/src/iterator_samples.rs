use jiter::{Jiter, JiterError, Peek};
use merde::{CowStr, Map, Value};

pub(crate) fn jiter_to_value<'j>(
    src: &'j str,
    iter: &mut Jiter<'j>,
) -> Result<Value<'j>, JiterError> {
    let peek = iter.peek()?;
    jiter_to_value_with_peek(src, peek, iter)
}

pub(crate) fn jiter_to_value_with_peek<'j>(
    src: &'j str,
    peek: Peek,
    iter: &mut Jiter<'j>,
) -> Result<Value<'j>, JiterError> {
    Ok(match peek {
        Peek::Null => Value::Null,
        Peek::True => Value::Bool(true),
        Peek::False => Value::Bool(false),
        Peek::Minus => unimplemented!(),
        Peek::Infinity => Value::Float(f64::INFINITY),
        Peek::NaN => Value::Float(f64::NAN),
        Peek::String => {
            let s = iter.known_str()?;
            Value::Str(cowify(src, s))
        }
        Peek::Array => {
            let mut arr = Vec::new();
            let mut next = iter.known_array()?;
            while let Some(peek) = next {
                arr.push(jiter_to_value_with_peek(src, peek, iter)?);
                next = iter.array_step()?;
            }
            Value::Array(arr.into())
        }
        Peek::Object => {
            let mut obj = Map::new();
            let mut next = iter.known_object()?;
            while let Some(key) = next {
                let key = cowify(src, key);
                let value = jiter_to_value_with_peek(src, iter.peek()?, iter)?;
                obj.insert(key, value);
                next = iter.next_key()?;
            }
            Value::Map(obj.into())
        }
        p if p.is_num() => {
            if let Ok(i) = iter.next_int() {
                match i {
                    jiter::NumberInt::Int(i) => Value::Int(i),
                    jiter::NumberInt::BigInt(_) => {
                        unimplemented!("BigInt")
                    }
                }
            } else if let Ok(f) = iter.next_float() {
                Value::Float(f)
            } else {
                unreachable!("not an int, not a float!")
            }
        }
        _ => unimplemented!("peek {:?}", peek),
    })
}

fn cowify<'j>(src: &'j str, s: &str) -> CowStr<'j> {
    if src.as_bytes().as_ptr_range().contains(&s.as_ptr()) {
        let start = unsafe { s.as_ptr().offset_from(src.as_ptr()) };
        let start: usize = start.try_into().unwrap();
        CowStr::Borrowed(&src[start..][..s.len()])
    } else {
        CowStr::Owned(s.into())
    }
}

#[test]
fn test_cowify() {
    let src = "That's a subset!";
    let s = &src[4..8];
    assert_eq!(cowify(src, s), CowStr::Borrowed(s));

    let src = "Not a subset";
    let s = "indeed not";
    assert_eq!(cowify(src, s), CowStr::Owned(s.into()));
}

#[test]
fn test_jiter_to_value() {
    let src = r#"
    {
        "name": "John Doe",
        "age": 42,
        "address": {
            "street": "123 Main St",
            "city": "Anytown",
            "state": "CA",
            "zip": 12345
        },
        "friends": [
            "Alice",
            "Bob",
            "Charlie"
        ]
    }
    "#;

    let mut iter = Jiter::new(src.as_bytes());
    let value = jiter_to_value(src, &mut iter).unwrap();
    assert_eq!(
        value,
        Value::Map({
            let mut map = Map::new();
            map.insert("name", Value::Str(CowStr::from("John Doe")));
            map.insert("age", Value::Int(42));
            map.insert(
                "address",
                Value::Map({
                    let mut map = Map::new();
                    map.insert("street", Value::Str(CowStr::from("123 Main St")));
                    map.insert("city", Value::Str(CowStr::from("Anytown")));
                    map.insert("state", Value::Str(CowStr::from("CA")));
                    map.insert("zip", Value::Int(12345));
                    map
                }),
            );
            map.insert(
                "friends",
                Value::Array(
                    vec![
                        Value::from("Alice"),
                        Value::from("Bob"),
                        Value::from("Charlie"),
                    ]
                    .into(),
                ),
            );
            map
        })
    );
}
