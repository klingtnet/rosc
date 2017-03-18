use errors;
use std::result;

/// see OSC Type Tag String: [OSC Spec. 1.0](http://opensoundcontrol.org/spec-1_0)
/// padding: zero bytes (n*4)
#[derive(Clone,Debug, PartialEq)]
pub enum OscType {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
    // use struct for time tag to avoid destructuring
    Time(u32, u32),
    Long(i64),
    Double(f64),
    Char(char),
    Color(OscColor),
    Midi(OscMidiMessage),
    Bool(bool),
    Nil,
    Inf,
}

/// Represents the parts of a Midi message. Mainly used for
/// tunneling midi over a network using the OSC protocol.
#[derive(Clone,Debug, PartialEq)]
pub struct OscMidiMessage {
    pub port: u8,
    pub status: u8,
    pub data1: u8, // maybe use an enum for data?
    pub data2: u8,
}

/// An *osc packet* can contain an *osc message* or a bundle of nested messages
/// which is called *osc bundle*.
#[derive(Clone,Debug, PartialEq)]
pub enum OscPacket {
    Message(OscMessage),
    Bundle(OscBundle),
}

/// An OSC message consists of an address and
/// zero or more arguments. The address should
/// specify an element of your Instrument (or whatever
/// you want to control with OSC) and the arguments
/// are used to set properties of the element to the
/// respective values.
#[derive(Clone,Debug, PartialEq)]
pub struct OscMessage {
    pub addr: String,
    pub args: Option<Vec<OscType>>,
}

/// An OSC bundle contains zero or more OSC packets
/// and a time tag. The contained packets *should* be
/// applied at the given time tag.
#[derive(Clone,Debug, PartialEq)]
pub struct OscBundle {
    pub timetag: OscType,
    pub content: Vec<OscPacket>,
}

/// An RGBA color.
#[derive(Clone,Debug, PartialEq)]
pub struct OscColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

pub type Result<T> = result::Result<T, errors::OscError>;

impl From<String> for OscMessage {
    fn from(s: String) -> OscMessage {
        OscMessage {
            addr: s,
            args: None
        }
    }
}
impl<'a> From<&'a str> for OscMessage {
    fn from(s: &str) -> OscMessage {
        OscMessage {
            addr: s.to_string(),
            args: None
        }
    }
}
