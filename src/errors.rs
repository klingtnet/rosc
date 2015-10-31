use std::fmt;

type OscResult<T> = Result<T, OscError>;

pub enum OscError {
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
