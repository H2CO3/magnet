//! Errors potentially happening while `#[derive]`ing `BsonSchema`.

use std::fmt;
use std::error;
use std::result;
use std::ops::Deref;
use syn::synom::ParseError;

/// An error that potentially happens while `#[derive]`ing `BsonSchema`.
#[derive(Debug)]
pub struct Error {
    /// The error message.
    message: String,
    /// The underlying error, if any.
    cause: Option<Box<error::Error>>,
}

impl Error {
    /// Creates an `Error` instance with the specified message.
    pub fn new<T: Into<String>>(message: T) -> Self {
        Error {
            message: message.into(),
            cause: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.cause {
            Some(ref cause) => write!(f, "{}: {}", self.message, cause),
            None => self.message.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&error::Error> {
        self.cause.as_ref().map(Deref::deref)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error {
            message: String::from("could not parse derive input"),
            cause: Some(Box::new(error)),
        }
    }
}

/// Convenience type alias for a result that holds a `magnet_derive::Error` value.
pub type Result<T> = result::Result<T, Error>;
