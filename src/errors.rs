use byteorder;
use std::{io, string};

#[derive(Debug)]
pub enum OscError {
    StringError(string::FromUtf8Error),
    ReadError(io::Error),
    ByteOrderError(byteorder::Error),
    BadPacket(&'static str),
    BadAddress(&'static str),
    BadMessage(&'static str),
    BadString(&'static str),
    BadArg(String),
    BadBundle,
    Unimplemented,
}
