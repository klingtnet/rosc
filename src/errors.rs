use byteorder;
use std::{fmt, io, string};

#[derive(Debug)]
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
