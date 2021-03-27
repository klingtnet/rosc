use std::{error, fmt, io, string};

/// Represents errors returned by `decode` or `encode`.
#[derive(Debug)]
pub enum OscError {
    StringError(string::FromUtf8Error),
    ReadError(io::Error),
    BadPacket(&'static str),
    BadAddress(&'static str),
    BadMessage(&'static str),
    BadString(&'static str),
    BadArg(String),
    BadBundle(String),
    Unimplemented,
}

impl fmt::Display for OscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OscError::StringError(err) => write!(f, "reading OSC string as utf-8: {}", err),
            OscError::ReadError(err) => write!(f, "reading from buffer: {}", err),
            OscError::BadPacket(msg) => write!(f, "{}", msg),
            OscError::BadAddress(msg) => write!(f, "{}", msg),
            OscError::BadMessage(msg) => write!(f, "bad OSC message: {}", msg),
            OscError::BadString(msg) => write!(f, "bad OSC string: {}", msg),
            OscError::BadArg(msg) => write!(f, "bad OSC argument: {}", msg),
            OscError::BadBundle(msg) => write!(f, "bad OSC bundle: {}", msg),
            OscError::Unimplemented => write!(f, "unimplemented"),
        }
    }
}

impl error::Error for OscError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            OscError::StringError(ref err) => Some(err),
            OscError::ReadError(ref err) => Some(err),
            _ => None,
        }
    }
}
