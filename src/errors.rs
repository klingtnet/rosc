use byteorder;
use std::{fmt, io, string};

pub enum OscError {
    StringError(string::FromUtf8Error),
    ReadError(io::Error),
    ByteOrderError(byteorder::Error),
    BadOscPacket(String),
    BadOscAddress(String),
    BadOscMessage(String),
    BadOscString(String),
    BadOscArg(String),
    BadOscBundle,
}

impl fmt::Display for OscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", "TODO"),
        }
    }
}
