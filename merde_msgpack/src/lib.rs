#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use merde_core::{
    Deserialize, DeserializeOwned, Deserializer, DynDeserializerExt, Event, IntoStatic, MapStart,
    MerdeError, MetastackExt,
};

/// A MessagePack deserializer, that implements [`merde_core::Deserializer`].
pub struct MsgpackDeserializer<'s> {
    source: &'s [u8],
    offset: usize,
    stack: Vec<StackItem>,
    starter: Option<Event<'s>>,
}

#[derive(Debug)]
enum StackItem {
    Array(usize),
    Map(usize),
}

impl<'s> MsgpackDeserializer<'s> {
    /// Construct a new MessagePack deserializer
    pub fn new(source: &'s [u8]) -> Self {
        Self {
            source,
            offset: 0,
            stack: Vec::new(),
            starter: None,
        }
    }
}

impl std::fmt::Debug for MsgpackDeserializer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MsgpackDeserializer")
            .field("source_len", &self.source.len())
            .field("stack", &self.stack)
            .finish()
    }
}

impl<'s> Deserializer<'s> for MsgpackDeserializer<'s> {
    async fn next(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        if let Some(ev) = self.starter.take() {
            return Ok(ev);
        }

        if let Some(stack_item) = self.stack.last_mut() {
            match stack_item {
                StackItem::Array(count) => {
                    if *count == 0 {
                        self.stack.pop();
                        return Ok(Event::ArrayEnd);
                    }
                    *count -= 1;
                }
                StackItem::Map(count) => {
                    if *count == 0 {
                        self.stack.pop();
                        return Ok(Event::MapEnd);
                    }
                    *count -= 1;
                }
            }
        }

        if self.offset >= self.source.len() {
            return Err(MerdeError::eof());
        }

        let byte = self.source[self.offset];
        self.offset += 1;

        match byte {
            0xc0 => Ok(Event::Null),
            0xc2 => Ok(Event::Bool(false)),
            0xc3 => Ok(Event::Bool(true)),
            0xc4 => self.read_bytes_8(),
            0xc5 => self.read_bytes_16(),
            0xc6 => self.read_bytes_32(),
            0xcc => self.read_u8().map(|v| Event::U64(v as u64)),
            0xcd => self.read_u16().map(|v| Event::U64(v as u64)),
            0xce => self.read_u32().map(|v| Event::U64(v as u64)),
            0xcf => self.read_u64().map(Event::U64),
            0xd0 => self.read_i8().map(|v| Event::I64(v as i64)),
            0xd1 => self.read_i16().map(|v| Event::I64(v as i64)),
            0xd2 => self.read_i32().map(|v| Event::I64(v as i64)),
            0xd3 => self.read_i64().map(Event::I64),
            0xca => self.read_f32().map(|v| Event::F64(v as f64)),
            0xcb => self.read_f64().map(Event::F64),
            0xa0..=0xbf => {
                let len = (byte & 0x1f) as usize;
                self.read_str(len)
            }
            0xd9 => self.read_str_8(),
            0xda => self.read_str_16(),
            0xdb => self.read_str_32(),
            0x90..=0x9f => {
                let len = (byte & 0x0f) as usize;
                self.stack.push(StackItem::Array(len));
                Ok(Event::ArrayStart(merde_core::ArrayStart {
                    size_hint: Some(len),
                }))
            }
            0xdc => self.read_array_16(),
            0xdd => self.read_array_32(),
            0x80..=0x8f => {
                let len = (byte & 0x0f) as usize;
                self.stack.push(StackItem::Map(len * 2));
                Ok(Event::MapStart(MapStart {
                    size_hint: Some(len as _),
                }))
            }
            0xde => {
                let len = self.read_u16()?;
                self.stack.push(StackItem::Map(len as usize * 2));
                Ok(Event::MapStart(MapStart {
                    size_hint: Some(len as _),
                }))
            }
            0xdf => {
                let len = self.read_u32()?;
                self.stack.push(StackItem::Map(len as usize * 2));
                Ok(Event::MapStart(MapStart {
                    size_hint: Some(len as _),
                }))
            }
            0x00..=0x7f => Ok(Event::U64(byte as u64)),
            0xe0..=0xff => Ok(Event::I64((byte as i8) as i64)),
            0xd4..=0xd8 | 0xc7..=0xc9 => Err(MerdeError::BinaryParsingError {
                format: "msgpack",
                message: format!("unsupported extension type 0x{byte:02x}"),
            }),
            _ => Err(MerdeError::BinaryParsingError {
                format: "msgpack",
                message: format!("unsupported type 0x{byte:02x}"),
            }),
        }
    }

    fn put_back(&mut self, event: Event<'s>) -> Result<(), MerdeError<'s>> {
        self.starter = Some(event);
        Ok(())
    }
}

impl<'s> MsgpackDeserializer<'s> {
    fn read_u8(&mut self) -> Result<u8, MerdeError<'s>> {
        if self.offset + 1 > self.source.len() {
            return Err(MerdeError::eof());
        }
        let value = self.source[self.offset];
        self.offset += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, MerdeError<'s>> {
        if self.offset + 2 > self.source.len() {
            return Err(MerdeError::eof());
        }
        let value = u16::from_be_bytes([self.source[self.offset], self.source[self.offset + 1]]);
        self.offset += 2;
        Ok(value)
    }

    fn read_u32(&mut self) -> Result<u32, MerdeError<'s>> {
        if self.offset + 4 > self.source.len() {
            return Err(MerdeError::eof());
        }
        let value = u32::from_be_bytes([
            self.source[self.offset],
            self.source[self.offset + 1],
            self.source[self.offset + 2],
            self.source[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(value)
    }

    fn read_u64(&mut self) -> Result<u64, MerdeError<'s>> {
        if self.offset + 8 > self.source.len() {
            return Err(MerdeError::eof());
        }
        let value = u64::from_be_bytes([
            self.source[self.offset],
            self.source[self.offset + 1],
            self.source[self.offset + 2],
            self.source[self.offset + 3],
            self.source[self.offset + 4],
            self.source[self.offset + 5],
            self.source[self.offset + 6],
            self.source[self.offset + 7],
        ]);
        self.offset += 8;
        Ok(value)
    }

    fn read_i8(&mut self) -> Result<i8, MerdeError<'s>> {
        self.read_u8().map(|v| v as i8)
    }

    fn read_i16(&mut self) -> Result<i16, MerdeError<'s>> {
        self.read_u16().map(|v| v as i16)
    }

    fn read_i32(&mut self) -> Result<i32, MerdeError<'s>> {
        self.read_u32().map(|v| v as i32)
    }

    fn read_i64(&mut self) -> Result<i64, MerdeError<'s>> {
        self.read_u64().map(|v| v as i64)
    }

    fn read_f32(&mut self) -> Result<f32, MerdeError<'s>> {
        self.read_u32().map(f32::from_bits)
    }

    fn read_f64(&mut self) -> Result<f64, MerdeError<'s>> {
        self.read_u64().map(f64::from_bits)
    }

    fn read_str(&mut self, len: usize) -> Result<Event<'s>, MerdeError<'s>> {
        if self.offset + len > self.source.len() {
            return Err(MerdeError::eof());
        }
        let s = std::str::from_utf8(&self.source[self.offset..self.offset + len])?;
        self.offset += len;
        Ok(Event::Str(s.into()))
    }

    fn read_str_8(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u8()? as usize;
        self.read_str(len)
    }

    fn read_str_16(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u16()? as usize;
        self.read_str(len)
    }

    fn read_str_32(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u32()? as usize;
        self.read_str(len)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Event<'s>, MerdeError<'s>> {
        if self.offset + len > self.source.len() {
            return Err(MerdeError::eof());
        }
        let bytes = &self.source[self.offset..self.offset + len];
        self.offset += len;
        Ok(Event::Bytes(bytes.into()))
    }

    fn read_bytes_8(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u8()? as usize;
        self.read_bytes(len)
    }

    fn read_bytes_16(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u16()? as usize;
        self.read_bytes(len)
    }

    fn read_bytes_32(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u32()? as usize;
        self.read_bytes(len)
    }

    fn read_array_16(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u16()? as usize;
        self.stack.push(StackItem::Array(len));
        Ok(Event::ArrayStart(merde_core::ArrayStart {
            size_hint: Some(len),
        }))
    }

    fn read_array_32(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        let len = self.read_u32()? as usize;
        self.stack.push(StackItem::Array(len));
        Ok(Event::ArrayStart(merde_core::ArrayStart {
            size_hint: Some(len),
        }))
    }
}

/// Deserialize an instance of type `T` from a byte slice of MessagePack data.
pub fn from_slice<'s, T>(slice: &'s [u8]) -> Result<T, MerdeError<'s>>
where
    T: Deserialize<'s>,
{
    let mut deser = MsgpackDeserializer::new(slice);
    deser.deserialize_sync::<T>()
}

/// Deserialize an instance of type `T` from a byte slice of MessagePack data,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_slice_owned<T>(slice: &[u8]) -> Result<<T as IntoStatic>::Output, MerdeError<'_>>
where
    T: DeserializeOwned,
{
    let mut deser = MsgpackDeserializer::new(slice);
    T::deserialize_owned(&mut deser).run_sync_with_metastack()
}

#[cfg(test)]
mod tests {
    use merde_core::Array;
    use merde_core::DynDeserializerExt;
    use merde_core::Value;
    use merde_loggingserializer::LoggingDeserializer;

    // cf. `testdata-maker/src/main.rs`
    // regen with `just regen`
    static TEST_INPUT: &[u8] = include_bytes!("../testdata/test.msgpack");

    #[test]
    fn test_deserialize() {
        let deser = super::MsgpackDeserializer::new(TEST_INPUT);
        let mut deser = LoggingDeserializer::new(deser);

        let value = deser.deserialize_sync::<merde_core::Value>().unwrap();

        let array = value.as_array().unwrap();
        let mut iter = array.iter();

        assert_eq!(iter.next().unwrap(), &Value::Null);
        assert_eq!(iter.next().unwrap(), &Value::Bool(false));
        assert_eq!(iter.next().unwrap(), &Value::Bool(true));
        assert_eq!(iter.next().unwrap().as_u64().unwrap(), 42);
        assert_eq!(iter.next().unwrap().as_i64().unwrap(), -123);
        assert_eq!(iter.next().unwrap().as_u64().unwrap(), 1000000);
        assert_eq!(iter.next().unwrap().as_i64().unwrap(), -9876543210);
        assert_eq!(iter.next().unwrap().as_u64().unwrap(), 18446744073709551615);
        assert!((iter.next().unwrap().as_f64().unwrap() - 1.23456).abs() < 1e-5);
        assert_eq!(iter.next().unwrap().as_f64().unwrap(), 0.0);
        let value = iter.next().unwrap();
        assert!(value.as_f64().unwrap().is_infinite());
        assert!(value.as_f64().unwrap().is_sign_positive());
        let value = iter.next().unwrap();
        assert!(value.as_f64().unwrap().is_infinite());
        assert!(value.as_f64().unwrap().is_sign_negative());
        assert_eq!(iter.next().unwrap().as_f64().unwrap(), f32::MIN as f64);
        assert_eq!(iter.next().unwrap().as_f64().unwrap(), f32::MAX as f64);
        assert!((iter.next().unwrap().as_f64().unwrap() - 1.23456789).abs() < 1e-8);
        assert_eq!(iter.next().unwrap().as_f64().unwrap(), 0.0);
        let value = iter.next().unwrap();
        assert!(value.as_f64().unwrap().is_infinite());
        assert!(value.as_f64().unwrap().is_sign_positive());
        let value = iter.next().unwrap();
        assert!(value.as_f64().unwrap().is_infinite());
        assert!(value.as_f64().unwrap().is_sign_negative());
        assert_eq!(iter.next().unwrap().as_f64().unwrap(), f64::MIN);
        assert_eq!(iter.next().unwrap().as_f64().unwrap(), f64::MAX);
        assert!((iter.next().unwrap().as_f64().unwrap() - 1e-100).abs() < 1e-101);
        assert!((iter.next().unwrap().as_f64().unwrap() - 1e100).abs() < 1e99);
        assert_eq!(
            iter.next().unwrap().as_str().unwrap().as_ref(),
            "Hello, MessagePack!"
        );
        assert_eq!(iter.next().unwrap().as_bytes().unwrap(), &[][..]);
        assert_eq!(
            iter.next().unwrap().as_bytes().unwrap(),
            &[0xDE, 0xAD, 0xBE, 0xEF][..]
        );
        assert_eq!(
            iter.next().unwrap().as_bytes().unwrap(),
            &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]
        );
        assert_eq!(iter.next().unwrap().as_bytes().unwrap(), &[0xFF; 256][..]);
        assert!(iter.next().unwrap().as_array().unwrap().is_empty());
        assert_eq!(
            iter.next().unwrap().as_array().unwrap(),
            &Array(vec![Value::Null, Value::Bool(true)])
        );

        let map = iter.next().unwrap().as_map().unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&"key1".into()).unwrap().as_u64().unwrap(), 1);
        assert!((map.get(&"key2".into()).unwrap().as_f64().unwrap() - 2.7118).abs() < 1e-4);

        assert!(iter.next().unwrap().as_map().unwrap().is_empty());
    }
}
