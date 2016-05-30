use std::{io, string};

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
