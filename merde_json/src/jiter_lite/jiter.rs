use crate::jiter_lite as jiter;

use jiter::errors::{json_error, JiterError, JsonError, JsonType};
use jiter::number_decoder::{NumberAny, NumberFloat};
use jiter::parse::{Parser, Peek};
use jiter::string_decoder::{StringDecoder, Tape};

pub type JiterResult<T> = Result<T, JiterError>;

/// A JSON iterator.
#[derive(Debug)]
pub struct Jiter<'j> {
    data: &'j [u8],
    parser: Parser<'j>,
    tape: Tape,
    allow_inf_nan: bool,
    allow_partial_strings: bool,
}

impl Clone for Jiter<'_> {
    /// Clone a `Jiter`. Like the default implementation, but a new empty `tape` is used.
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            parser: self.parser.clone(),
            tape: Tape::default(),
            allow_inf_nan: self.allow_inf_nan,
            allow_partial_strings: self.allow_partial_strings,
        }
    }
}

impl<'j> Jiter<'j> {
    /// Constructs a new `Jiter`.
    ///
    /// # Arguments
    /// - `data`: The JSON data to be parsed.
    /// - `allow_inf_nan`: Whether to allow `NaN`, `Infinity` and `-Infinity` as numbers.
    pub fn new(data: &'j [u8]) -> Self {
        Self {
            data,
            parser: Parser::new(data),
            tape: Tape::default(),
            allow_inf_nan: false,
            allow_partial_strings: false,
        }
    }

    /// Peek at the next JSON value without consuming it.
    pub fn peek(&mut self) -> JiterResult<Peek> {
        self.parser.peek().map_err(Into::into)
    }

    /// Knowing the next value is `null`, consume it.
    pub fn known_null(&mut self) -> JiterResult<()> {
        self.parser.consume_null()?;
        Ok(())
    }

    /// Knowing the next value is `true` or `false`, parse it.
    pub fn known_bool(&mut self, peek: Peek) -> JiterResult<bool> {
        match peek {
            Peek::True => {
                self.parser.consume_true()?;
                Ok(true)
            }
            Peek::False => {
                self.parser.consume_false()?;
                Ok(false)
            }
            _ => Err(self.wrong_type(JsonType::Bool, peek)),
        }
    }

    /// Knowing the next value is a float, parse it.
    pub fn known_float(&mut self, peek: Peek) -> JiterResult<f64> {
        self.parser
            .consume_number::<NumberFloat>(peek.into_inner(), self.allow_inf_nan)
            .map_err(|e| self.maybe_number_error(e, JsonType::Float, peek))
    }

    /// Knowing the next value is a string, parse it.
    pub fn known_str(&mut self) -> JiterResult<&str> {
        match self
            .parser
            .consume_string::<StringDecoder>(&mut self.tape, self.allow_partial_strings)
        {
            Ok(output) => Ok(output.as_str()),
            Err(e) => Err(e.into()),
        }
    }

    /// Assuming the next value is an array, peek at the first value.
    pub fn known_array(&mut self) -> JiterResult<Option<Peek>> {
        self.parser.array_first().map_err(Into::into)
    }

    /// Peek at the next value in an array.
    pub fn array_step(&mut self) -> JiterResult<Option<Peek>> {
        self.parser.array_step().map_err(Into::into)
    }

    /// Assuming the next value is an object, conssume the first key and return bytes from the original JSON data.
    pub fn known_object(&mut self) -> JiterResult<Option<&str>> {
        let op_str = self.parser.object_first::<StringDecoder>(&mut self.tape)?;
        Ok(op_str.map(|s| s.as_str()))
    }

    /// Get the next key in an object, or `None` if there are no more keys.
    pub fn next_key(&mut self) -> JiterResult<Option<&str>> {
        let strs = self.parser.object_step::<StringDecoder>(&mut self.tape)?;
        Ok(strs.map(|s| s.as_str()))
    }

    fn wrong_type(&self, expected: JsonType, peek: Peek) -> JiterError {
        match peek {
            Peek::True | Peek::False => {
                JiterError::wrong_type(expected, JsonType::Bool, self.parser.index)
            }
            Peek::Null => JiterError::wrong_type(expected, JsonType::Null, self.parser.index),
            Peek::String => JiterError::wrong_type(expected, JsonType::String, self.parser.index),
            Peek::Array => JiterError::wrong_type(expected, JsonType::Array, self.parser.index),
            Peek::Object => JiterError::wrong_type(expected, JsonType::Object, self.parser.index),
            _ if peek.is_num() => self.wrong_num(peek.into_inner(), expected),
            _ => json_error!(ExpectedSomeValue, self.parser.index).into(),
        }
    }

    fn wrong_num(&self, first: u8, expected: JsonType) -> JiterError {
        let mut parser2 = self.parser.clone();
        let actual = match parser2.consume_number::<NumberAny>(first, self.allow_inf_nan) {
            Ok(NumberAny::Int { .. }) => JsonType::Int,
            Ok(NumberAny::Float { .. }) => JsonType::Float,
            Err(e) => return e.into(),
        };
        JiterError::wrong_type(expected, actual, self.parser.index)
    }

    fn maybe_number_error(&self, e: JsonError, expected: JsonType, peek: Peek) -> JiterError {
        if peek.is_num() {
            e.into()
        } else {
            self.wrong_type(expected, peek)
        }
    }
}
