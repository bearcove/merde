use jiter::{Jiter, JiterError, Peek};
use merde::{CowStr, LazyIndexMap, Value};

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
            // fun trick! we try to figure out if `s` is taken from `src`.
            // if yes, we can borrow. if no, we need to allocate.
            let range = src.as_bytes().as_ptr_range();
            if range.contains(&s.as_ptr()) {
                // okay, cool! let's transmute the lifetime away I guess.
                // the public API of `jiter` is just not flexible enough for this,
                // I wish they'd expose `Parser` or something.
                Value::String(CowStr::Borrowed(unsafe { std::mem::transmute(s) }))
            } else {
                // nope, this was written to the tape, let's copy
                Value::String(CowStr::Owned(s.into()))
            }
        }
        Peek::Array => {
            let mut arr = Vec::new();
            let mut next = iter.known_array()?;
            while let Some(peek) = next {
                arr.push(jiter_to_value_with_peek(src, peek, iter)?);
                next = iter.array_step()?;
            }
            Value::Array(arr)
        }
        Peek::Object => {
            let mut obj = LazyIndexMap::new();
            let mut next = iter.known_object()?;
            while let Some(key) = next {
                let key = cowify(src, key);
                let value = jiter_to_value_with_peek(src, iter.peek()?, iter)?;
                obj.insert(key, value);
                next = iter.next_key()?;
            }
            Value::Object(obj)
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
        }
    }
    "#;

    let mut iter = Jiter::new(src.as_bytes());
    let value = jiter_to_value(src, &mut iter).unwrap();
    assert_eq!(
        value,
        Value::Object({
            let mut map = LazyIndexMap::new();
            map.insert(
                CowStr::from("name"),
                Value::String(CowStr::Borrowed("John Doe")),
            );
            map.insert(CowStr::from("age"), Value::Int(42));
            map.insert(
                CowStr::from("address"),
                Value::Object({
                    let mut map = LazyIndexMap::new();
                    map.insert(
                        CowStr::from("street"),
                        Value::String(CowStr::Borrowed("123 Main St")),
                    );
                    map.insert(
                        CowStr::from("city"),
                        Value::String(CowStr::Borrowed("Anytown")),
                    );
                    map.insert(CowStr::from("state"), Value::String(CowStr::Borrowed("CA")));
                    map.insert(CowStr::from("zip"), Value::Int(12345));
                    map
                }),
            );
            map
        })
    );
}
