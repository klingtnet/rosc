use std::{fmt, io, string};

pub enum OscError {
    StringError(string::FromUtf8Error),
    ReadError(io::Error),
    BadOscPacket(String),
    BadOscAddress(String),
    BadOscMessage(String),
    BadOscString(String),
    BadOscBundle,
}

impl fmt::Display for OscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{}", "TODO"),
        }
    }
}
