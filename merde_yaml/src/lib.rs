#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::str::Chars;

use merde_core::{
    ArrayStart, Deserialize, DeserializeOwned, Deserializer, DynDeserializerExt, Event, MapStart,
    MerdeError,
};
use yaml_rust2::{parser::Parser, scanner::TScalarStyle};

/// A YAML deserializer, that implements [`merde_core::Deserializer`].
pub struct YamlDeserializer<'s> {
    source: &'s str,
    parser: Parser<Chars<'s>>,
    starter: Option<Event<'s>>,
}

impl std::fmt::Debug for YamlDeserializer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YamlDeserializer")
            .field("source_len", &self.source)
            .finish()
    }
}

impl<'s> YamlDeserializer<'s> {
    /// Construct a new YAML deserializer
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            parser: Parser::new_from_str(source),
            starter: None,
        }
    }
}

impl<'s> Deserializer<'s> for YamlDeserializer<'s> {
    async fn next(&mut self) -> Result<Event<'s>, MerdeError<'s>> {
        loop {
            if let Some(starter) = self.starter.take() {
                return Ok(starter);
            }

            let (ev, _marker) = match self.parser.next_token() {
                Ok(ev) => ev,
                Err(e) => {
                    return Err(MerdeError::StringParsingError {
                        format: "yaml",
                        source: self.source.into(),
                        index: 0,
                        message: e.to_string(),
                    });
                }
            };

            use yaml_rust2::Event as YEvent;

            let res = match ev {
                YEvent::StreamEnd => Err(MerdeError::eof()),
                YEvent::Nothing
                | YEvent::StreamStart
                | YEvent::DocumentStart
                | YEvent::DocumentEnd => {
                    // ignore those
                    continue;
                }
                YEvent::Alias(_) => {
                    todo!("aliases?")
                }
                YEvent::Scalar(s, style, _anchor_id, tag) => {
                    if style != TScalarStyle::Plain {
                        Ok(Event::Str(s.into()))
                    } else if let Some(tag) = tag {
                        if tag.handle == "tag:yaml.org,2002:" {
                            // TODO: use faster int/float parsers
                            match tag.suffix.as_ref() {
                                "bool" => match s.parse::<bool>() {
                                    Ok(v) => Ok(Event::Bool(v)),
                                    Err(_) => Err(MerdeError::StringParsingError {
                                        format: "yaml",
                                        source: self.source.into(),
                                        index: 0,
                                        message: "failed to parse bool".to_string(),
                                    }),
                                },
                                "int" => match s.parse::<i64>() {
                                    Ok(v) => Ok(Event::I64(v)),
                                    Err(_) => Err(MerdeError::StringParsingError {
                                        format: "yaml",
                                        source: self.source.into(),
                                        index: 0,
                                        message: "failed to parse int".to_string(),
                                    }),
                                },
                                "float" => match s.parse::<f64>() {
                                    Ok(v) => Ok(Event::F64(v)),
                                    Err(_) => Err(MerdeError::StringParsingError {
                                        format: "yaml",
                                        source: self.source.into(),
                                        index: 0,
                                        message: "failed to parse float".to_string(),
                                    }),
                                },
                                "null" => match s.as_ref() {
                                    "~" | "null" => Ok(Event::Null),
                                    _ => Err(MerdeError::StringParsingError {
                                        format: "yaml",
                                        source: self.source.into(),
                                        index: 0,
                                        message: "failed to parse null".to_string(),
                                    }),
                                },
                                _ => Ok(Event::Str(s.into())),
                            }
                        } else {
                            Ok(Event::Str(s.into()))
                        }
                    } else {
                        // Datatype is not specified, try to infer
                        if let Ok(v) = s.parse::<bool>() {
                            Ok(Event::Bool(v))
                        } else if let Ok(v) = s.parse::<i64>() {
                            Ok(Event::I64(v))
                        } else if let Ok(v) = s.parse::<f64>() {
                            Ok(Event::F64(v))
                        } else if s == "~" || s == "null" {
                            Ok(Event::Null)
                        } else {
                            Ok(Event::Str(s.into()))
                        }
                    }
                }
                YEvent::SequenceStart(_, _tag) => {
                    Ok(Event::ArrayStart(ArrayStart { size_hint: None }))
                }
                YEvent::SequenceEnd => Ok(Event::ArrayEnd),
                YEvent::MappingStart(_, _tag) => Ok(Event::MapStart(MapStart { size_hint: None })),
                YEvent::MappingEnd => Ok(Event::MapEnd),
            };
            return res;
        }
    }

    fn put_back(&mut self, event: Event<'s>) -> Result<(), MerdeError<'s>> {
        if self.starter.is_some() {
            return Err(MerdeError::PutBackCalledTwice);
        }
        self.starter = Some(event);
        Ok(())
    }
}

/// Deserialize an instance of type `T` from a string of YAML text.
pub fn from_str<'s, T>(s: &'s str) -> Result<T, MerdeError<'s>>
where
    T: Deserialize<'s>,
{
    let mut deser = YamlDeserializer::new(s);
    deser.deserialize_sync::<T>()
}

/// Deserialize an instance of type `T` from a string of YAML text,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_str_owned<T>(s: &str) -> Result<T, MerdeError<'_>>
where
    T: DeserializeOwned,
{
    use merde_core::MetastackExt;
    let mut deser = YamlDeserializer::new(s);
    T::deserialize_owned(&mut deser).run_sync_with_metastack()
}
