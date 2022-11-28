use alloc::{
    fmt,
    string::{self, String},
};
use nom::error::{ErrorKind, FromExternalError, ParseError};
#[cfg(feature = "std")]
use std::error;

/// Represents errors returned by `decode` or `encode`.
#[derive(Debug)]
pub enum OscError {
    StringError(string::FromUtf8Error),
    ReadError(ErrorKind),
    BadChar(char),
    BadPacket(&'static str),
    BadMessage(&'static str),
    BadString(&'static str),
    BadArg(String),
    BadBundle(String),
    BadAddressPattern(String),
    BadAddress(String),
    RegexError(String),
    Unimplemented,
}

impl fmt::Display for OscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OscError::StringError(err) => write!(f, "reading OSC string as utf-8: {}", err),
            OscError::ReadError(kind) => write!(f, "error reading from buffer: {:?}", kind),
            OscError::BadChar(char) => write!(f, "parser error at char: {:?}", char),
            OscError::BadPacket(msg) => write!(f, "bad OSC packet: {}", msg),
            OscError::BadMessage(msg) => write!(f, "bad OSC message: {}", msg),
            OscError::BadString(msg) => write!(f, "bad OSC string: {}", msg),
            OscError::BadArg(msg) => write!(f, "bad OSC argument: {}", msg),
            OscError::BadBundle(msg) => write!(f, "bad OSC bundle: {}", msg),
            OscError::BadAddressPattern(msg) => write!(f, "bad OSC address pattern: {}", msg),
            OscError::BadAddress(msg) => write!(f, "bad OSC address: {}", msg),
            OscError::RegexError(msg) => write!(f, "OSC address pattern regex error: {}", msg),
            OscError::Unimplemented => write!(f, "unimplemented"),
        }
    }
}

impl<I> ParseError<I> for OscError {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Self::ReadError(kind)
    }
    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(_input: I, c: char) -> Self {
        Self::BadChar(c)
    }

    fn or(self, _other: Self) -> Self {
        self
    }
}

impl<I> FromExternalError<I, OscError> for OscError {
    fn from_external_error(_input: I, _kind: ErrorKind, e: OscError) -> Self {
        e
    }
}

#[cfg(feature = "std")]
impl error::Error for OscError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            OscError::StringError(ref err) => Some(err),
            _ => None,
        }
    }
}
