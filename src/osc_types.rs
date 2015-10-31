// see OSC Type Tag String: http://opensoundcontrol.org/spec-1_0
// padding: zero bytes (n*4)
pub enum OscType {
    OscInt(i32),
    OscFloat(f32),
    OscString(String), // padding
    OscBlob(Vec<u8>), // padding
    OscTimetag(u64),
    // nonstandard argument types
    // ignore them if not implemented
    OscInt64(i64), // big-endian
    OscDouble(u64),
    OscChar(u8), // padding
    OscColor(u32), // RGBA
    OscMidi(OscMidiType),
    OscTrue,
    OscFalse,
    OscNil,
    OscInf,
    OscArray(Vec<OscType>),
}

pub struct OscMidiType {
    port: u8,
    status: u8,
    data1: u8, // maybe use an enum for data?
    data2: u8,
}

/// An *osc packet* can contain an *osc message* or a bundle of nested messages
/// which is called *osc bundle*.
pub enum OscPacket {
    Message(OscMessage),
    Bundle(OscBundle),
}

pub struct OscMessage;
pub struct OscBundle;
