use rmpv::Value;
use std::fs::File;
use std::io::Write;

fn generate_test_messagepack() -> Vec<u8> {
    let value = Value::Array(vec![
        Value::Nil,
        Value::Boolean(false),
        Value::Boolean(true),
        Value::Integer(42.into()),
        Value::Integer((-123).into()),
        Value::Integer(1000000.into()),
        Value::Integer((-9876543210i64).into()),
        Value::Integer(18446744073709551615u64.into()),
        Value::F32(1.23456),
        Value::F32(0.0),
        Value::F32(f32::INFINITY),
        Value::F32(f32::NEG_INFINITY),
        Value::F32(f32::MIN),
        Value::F32(f32::MAX),
        Value::F64(1.23456789),
        Value::F64(0.0),
        Value::F64(f64::INFINITY),
        Value::F64(f64::NEG_INFINITY),
        Value::F64(f64::MIN),
        Value::F64(f64::MAX),
        Value::F64(1e-100),
        Value::F64(1e100),
        Value::String("Hello, MessagePack!".into()),
        Value::Binary(vec![]),
        Value::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        Value::Binary(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
        Value::Binary(vec![0xFF; 256]),
        Value::Array(vec![]),
        Value::Array(vec![Value::Nil, Value::Boolean(true)]),
        Value::Map(vec![
            (Value::String("key1".into()), Value::Integer(1.into())),
            (Value::String("key2".into()), Value::F64(2.7118)),
        ]),
        Value::Map(vec![]),
    ]);

    let mut buf = Vec::new();
    rmpv::encode::write_value(&mut buf, &value).unwrap();
    buf
}

fn main() {
    let encoded = generate_test_messagepack();
    let mut file = File::create("testdata/test.msgpack").unwrap();
    file.write_all(&encoded).unwrap();
}
