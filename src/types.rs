use errors;
use std::result;

// see OSC Type Tag String: http://opensoundcontrol.org/spec-1_0
// padding: zero bytes (n*4)

#[derive(Debug)]
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

#[derive(Debug)]
pub struct OscMidiMessage {
    pub port: u8,
    pub status: u8,
    pub data1: u8, // maybe use an enum for data?
    pub data2: u8,
}
impl PartialEq for OscMidiMessage {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port && self.status == other.status && self.data1 == other.data1 &&
        self.data2 == other.data2
    }
}

/// An *osc packet* can contain an *osc message* or a bundle of nested messages
/// which is called *osc bundle*.
#[derive(Debug)]
pub enum OscPacket {
    Message(OscMessage),
    Bundle(OscBundle),
}
impl PartialEq for OscPacket {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&OscPacket::Message(ref m1), &OscPacket::Message(ref m2)) => m1 == m2,
            (&OscPacket::Bundle(ref b1), &OscPacket::Bundle(ref b2)) => b1 == b2,
            (_, _) => false,
        }
    }
}

#[derive(Debug)]
pub struct OscMessage {
    pub addr: String,
    pub args: Option<Vec<OscType>>,
}
impl PartialEq for OscMessage {
    fn eq(&self, other: &Self) -> bool {
        (self.addr == other.addr) &&
        match (&self.args, &other.args) {
            (&Some(ref self_args), &Some(ref other_args)) => {
                self_args.iter()
                         .zip(other_args)
                         .map(|(x, y)| {
                             match (x, y) {
                                 (&OscType::Int(x), &OscType::Int(y)) => x == y,
                                 (&OscType::Long(x), &OscType::Long(y)) => x == y,
                                 (&OscType::Float(x), &OscType::Float(y)) => x == y,
                                 (&OscType::Double(x), &OscType::Double(y)) => x == y,
                                 (&OscType::String(ref x), &OscType::String(ref y)) => x == y,
                                 (&OscType::Blob(ref x), &OscType::Blob(ref y)) => x == y,
                                 (&OscType::Time(x_sec, x_frac), &OscType::Time(y_sec, y_frac)) =>
                                     (x_sec, x_frac) == (y_sec, y_frac),
                                 (&OscType::Char(x), &OscType::Char(y)) => x == y,
                                 (&OscType::Bool(x), &OscType::Bool(y)) => x == y,
                                 (&OscType::Midi(ref x), &OscType::Midi(ref y)) => x == y,
                                 (&OscType::Color(ref x), &OscType::Color(ref y)) => x == y,
                                 (&OscType::Inf, &OscType::Inf) => true,
                                 (&OscType::Nil, &OscType::Nil) => true,
                                 (_, _) => false,
                             }
                         })
                         .all(|x| x == true)
            }
            (&None, &None) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct OscBundle {
    pub timetag: OscType,
    pub content: Vec<OscPacket>,
}
impl PartialEq for OscBundle {
    fn eq(&self, other: &Self) -> bool {
        self.content.iter().zip(&other.content).map(|(x, y)| x == y).all(|x| x == true) &&
        match (&self.timetag, &other.timetag) {
            (&OscType::Time(self_sec, self_frac),
             &OscType::Time(other_sec, other_frac)) =>
                self_sec == other_sec && self_frac == other_frac,
            (_, _) => false,
        }
    }
}

#[derive(Debug)]
pub struct OscColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl PartialEq for OscColor {
    fn eq(&self, other: &Self) -> bool {
        self.red == other.red && self.green == other.green && self.blue == other.blue &&
        self.alpha == other.alpha
    }
}

pub type Result<T> = result::Result<T, errors::OscError>;
