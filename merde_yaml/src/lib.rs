#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::str::Chars;

use merde_core::{ArrayStart, Deserialize, DeserializeOwned, Deserializer, Event, MapStart};
use yaml_rust2::{parser::Parser, scanner::TScalarStyle, ScanError};

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

/// Unifies [`merde_core::MerdeError`], [`yaml_rust2::ScanError`], and our own parsing errors.
#[derive(Debug)]
pub enum MerdeYamlError<'s> {
    /// Most likely an error encountered when "destructuring" the JSON data.
    MerdeError(merde_core::MerdeError<'s>),

    /// Most likely a YAML syntax error
    ScanError(ScanError),

    /// For now, a type mismatch
    ParseError {
        /// the type we expected
        expected_type: &'static str,
    },

    /// EOF encountered while expecting a value
    Eof,
}

impl<'s> From<merde_core::MerdeError<'s>> for MerdeYamlError<'s> {
    fn from(e: merde_core::MerdeError<'s>) -> MerdeYamlError<'s> {
        MerdeYamlError::MerdeError(e)
    }
}

impl<'s> From<ScanError> for MerdeYamlError<'s> {
    fn from(e: ScanError) -> MerdeYamlError<'s> {
        MerdeYamlError::ScanError(e)
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
    type Error<'es> = MerdeYamlError<'es>;

    async fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>> {
        loop {
            if let Some(starter) = self.starter.take() {
                return Ok(starter);
            }

            let (ev, _marker) = match self.parser.next_token() {
                Ok(ev) => ev,
                Err(e) => {
                    // TODO: add location info, etc.
                    return Err(e.into());
                }
            };

            use yaml_rust2::Event as YEvent;

            let res = match ev {
                YEvent::StreamEnd => Err(MerdeYamlError::Eof),
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
                                    Err(_) => Err(MerdeYamlError::ParseError {
                                        expected_type: "bool",
                                    }),
                                },
                                "int" => match s.parse::<i64>() {
                                    Ok(v) => Ok(Event::I64(v)),
                                    Err(_) => Err(MerdeYamlError::ParseError {
                                        expected_type: "int",
                                    }),
                                },
                                "float" => match s.parse::<f64>() {
                                    Ok(v) => Ok(Event::F64(v)),
                                    Err(_) => Err(MerdeYamlError::ParseError {
                                        expected_type: "float",
                                    }),
                                },
                                "null" => match s.as_ref() {
                                    "~" | "null" => Ok(Event::Null),
                                    _ => Err(MerdeYamlError::ParseError {
                                        expected_type: "null",
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

    async fn t_starting_with<T: merde_core::Deserialize<'s>>(
        &mut self,
        starter: Option<Event<'s>>,
    ) -> Result<T, Self::Error<'s>> {
        if let Some(starter) = starter {
            if self.starter.is_some() {
                unreachable!("setting starter when it's already set? shouldn't happen")
            }
            self.starter = Some(starter);
        }

        // TODO: when too much stack space is used, stash this,
        // return Poll::Pending, to continue deserializing with
        // a shallower stack.

        // that's the whole trick — for now, we just recurse as usual
        T::deserialize(self).await
    }
}

/// Deserialize an instance of type `T` from a string of YAML text.
pub fn from_str<'s, T>(s: &'s str) -> Result<T, MerdeYamlError<'s>>
where
    T: Deserialize<'s>,
{
    let mut deser = YamlDeserializer::new(s);
    deser.deserialize_sync::<T>()
}

/// Deserialize an instance of type `T` from a string of YAML text,
/// and return its static variant e.g. (CowStr<'static>, etc.)
pub fn from_str_owned<T>(s: &str) -> Result<T, MerdeYamlError<'_>>
where
    T: DeserializeOwned,
{
    let mut deser = YamlDeserializer::new(s);
    T::deserialize_owned(&mut deser)
}
