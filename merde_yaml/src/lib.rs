use std::str::Chars;

use merde_core::{ArrayStart, Deserializer, Event};
use yaml_rust2::{parser::Parser, scanner::TScalarStyle, ScanError};

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

#[derive(Debug)]
pub enum MerdeYamlError<'s> {
    MerdeError(merde_core::MerdeError<'s>),
    ScanError(ScanError),
    ParseError { expected_type: &'static str },
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

    fn next(&mut self) -> Result<Event<'s>, Self::Error<'s>> {
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

        match ev {
            YEvent::Nothing
            | YEvent::StreamStart
            | YEvent::StreamEnd
            | YEvent::DocumentStart
            | YEvent::DocumentEnd => {
                // ignore those
                return self.next();
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
                                Ok(v) => Ok(Event::Int(v)),
                                Err(_) => Err(MerdeYamlError::ParseError {
                                    expected_type: "int",
                                }),
                            },
                            "float" => match s.parse::<f64>() {
                                Ok(v) => Ok(Event::Float(v)),
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
                        Ok(Event::Int(v))
                    } else if let Ok(v) = s.parse::<f64>() {
                        Ok(Event::Float(v))
                    } else if s == "~" || s == "null" {
                        Ok(Event::Null)
                    } else {
                        Ok(Event::Str(s.into()))
                    }
                }
            }
            YEvent::SequenceStart(_, _tag) => Ok(Event::ArrayStart(ArrayStart { size_hint: None })),
            YEvent::SequenceEnd => Ok(Event::ArrayEnd),
            YEvent::MappingStart(_, _tag) => Ok(Event::MapStart),
            YEvent::MappingEnd => Ok(Event::MapEnd),
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
