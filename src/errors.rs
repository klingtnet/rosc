use byteorder;
use std::{io, string};

#[derive(Debug)]
pub enum OscError {
    StringError(string::FromUtf8Error),
    ReadError(io::Error),
    ByteOrderError(byteorder::Error),
    BadPacket(String),
    BadAddress(String),
    BadMessage(String),
    BadString(String),
    BadArg(String),
    BadBundle,
}
