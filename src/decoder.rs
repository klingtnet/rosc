use types::{OscPacket, OscType, Result, OscMessage, OscMidiMessage, OscColor, OscBundle};
use errors::OscError;
use encoder;

use std::{io, char};
use std::io::{Read, BufRead};

use byteorder::{BigEndian, ReadBytesExt};

/// Common MTU size for ethernet
pub const MTU: usize = 1536;

/// Takes an byte slice as argument and returns an
/// OSC packet on success or an `OscError` if the slice
/// does not contain a valid OSC message.
pub fn decode(msg: &[u8]) -> Result<OscPacket> {
    if msg.len() == 0 {
        return Err(OscError::BadPacket("Empty packet."));
    }

    match msg[0] as char {
        '/' => decode_message(msg),
        '#' => decode_bundle(msg),
        _ => Err(OscError::BadPacket("Unknown message format.")),
    }
}

fn decode_message(msg: &[u8]) -> Result<OscPacket> {
    let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(msg);

    let addr: String = try!(read_osc_string(&mut cursor));
    let type_tags: String = try!(read_osc_string(&mut cursor));

    if type_tags.len() > 1 {
        let args: Vec<OscType> = try!(read_osc_args(&mut cursor, type_tags));

        Ok(OscPacket::Message(OscMessage {
            addr: addr,
            args: Some(args),
        }))
    } else {
        Ok(OscPacket::Message(OscMessage {
            addr: addr,
            args: None,
        }))
    }
}

fn decode_bundle(msg: &[u8]) -> Result<OscPacket> {
    let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(msg);

    let bundle_tag = try!(read_osc_string(&mut cursor));
    if bundle_tag != "#bundle" {
        return Err(OscError::BadBundle(format!("Wrong bundle specifier: {}", bundle_tag)));
    }

    let time_tag = try!(read_time_tag(&mut cursor));

    let mut bundle: Vec<OscPacket> = Vec::new();

    let mut elem_size = try!(read_bundle_element_size(&mut cursor));

    while msg.len() >= (cursor.position() as usize) + elem_size {
        let packet = try!(read_bundle_element(&mut cursor, elem_size));
        bundle.push(packet);

        if msg.len() == cursor.position() as usize {
            break;
        }

        elem_size = try!(read_bundle_element_size(&mut cursor));
    }

    Ok(OscPacket::Bundle(OscBundle {
        timetag: time_tag,
        content: bundle,
    }))
}

fn read_bundle_element_size(cursor: &mut io::Cursor<&[u8]>) -> Result<usize> {
    cursor.read_u32::<BigEndian>()
          .map(|size| size as usize)
          .map_err(OscError::ByteOrderError)
}

fn read_bundle_element(cursor: &mut io::Cursor<&[u8]>, elem_size: usize) -> Result<OscPacket> {
    let mut buf: Vec<u8> = Vec::with_capacity(elem_size);

    let mut handle = cursor.take(elem_size as u64);

    let cnt = try!(handle.read_to_end(&mut buf)
                         .map_err(OscError::ReadError));

    if cnt == elem_size {
        decode(&mut buf)
    } else {
        Err(OscError::BadBundle("Bundle shorter than expected!".to_string()))
    }
}

fn read_osc_string(cursor: &mut io::Cursor<&[u8]>) -> Result<String> {
    let mut str_buf: Vec<u8> = Vec::new();
    try!(cursor.read_until(0, &mut str_buf)
               .map_err(OscError::ReadError)); // ignore returned byte count
    pad_cursor(cursor);
    // convert to String and remove nul bytes
    String::from_utf8(str_buf)
        .map_err(|e| OscError::StringError(e))
        .map(|s| {
            s.trim_matches(0u8 as char)
             .to_string()
        })
}

fn read_osc_args(cursor: &mut io::Cursor<&[u8]>, raw_type_tags: String) -> Result<Vec<OscType>> {
    let type_tags: Vec<char> = raw_type_tags.chars()
                                            .skip(1)
                                            .collect();

    let mut args: Vec<OscType> = Vec::with_capacity(type_tags.len());
    for tag in type_tags {
        let arg: OscType = try!(read_osc_arg(cursor, tag));
        args.push(arg);
    }
    Ok(args)
}

fn read_osc_arg(cursor: &mut io::Cursor<&[u8]>, tag: char) -> Result<OscType> {
    match tag {
        'f' => {
            cursor.read_f32::<BigEndian>()
                  .map(OscType::Float)
                  .map_err(OscError::ByteOrderError)
        }
        'd' => {
            cursor.read_f64::<BigEndian>()
                  .map(OscType::Double)
                  .map_err(OscError::ByteOrderError)
        }
        'i' => {
            cursor.read_i32::<BigEndian>()
                  .map(OscType::Int)
                  .map_err(OscError::ByteOrderError)
        }
        'h' => {
            cursor.read_i64::<BigEndian>()
                  .map(OscType::Long)
                  .map_err(OscError::ByteOrderError)
        }
        's' => read_osc_string(cursor).map(|s| OscType::String(s)),
        't' => read_time_tag(cursor),
        'b' => read_blob(cursor),
        'r' => read_osc_color(cursor),
        'T' => Ok(OscType::Bool(true)),
        'F' => Ok(OscType::Bool(false)),
        'N' => Ok(OscType::Nil),
        'I' => Ok(OscType::Inf),
        'c' => read_char(cursor),
        'm' => read_midi_message(cursor),
        _ => Err(OscError::BadArg(format!("Type tag \"{}\" is not implemented!", tag))),
    }
}

fn read_char(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let opt_char = try!(cursor.read_u32::<BigEndian>()
                              .map(char::from_u32)
                              .map_err(OscError::ByteOrderError));
    match opt_char {
        Some(c) => Ok(OscType::Char(c)),
        None => Err(OscError::BadArg("Argument is not a char!".to_string())),
    }
}

fn read_blob(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let size: usize = try!(cursor.read_u32::<BigEndian>()
                                 .map_err(OscError::ByteOrderError)) as usize;
    let mut byte_buf: Vec<u8> = Vec::with_capacity(size);

    try!(cursor.take(size as u64)
               .read_to_end(&mut byte_buf)
               .map_err(OscError::ReadError));

    pad_cursor(cursor);

    Ok(OscType::Blob(byte_buf))
}

fn read_time_tag(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let date = try!(cursor.read_u32::<BigEndian>()
                          .map_err(OscError::ByteOrderError));
    let frac = try!(cursor.read_u32::<BigEndian>()
                          .map_err(OscError::ByteOrderError));

    Ok(OscType::Time(date, frac))
}

fn read_midi_message(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let mut buf: Vec<u8> = Vec::with_capacity(4);
    try!(cursor.take(4).read_to_end(&mut buf).map_err(OscError::ReadError));

    Ok(OscType::Midi(OscMidiMessage {
        port: buf[0],
        status: buf[1],
        data1: buf[2],
        data2: buf[3],
    }))

}

fn read_osc_color(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let mut buf: Vec<u8> = Vec::with_capacity(4);
    try!(cursor.take(4).read_to_end(&mut buf).map_err(OscError::ReadError));

    Ok(OscType::Color(OscColor {
        red: buf[0],
        green: buf[1],
        blue: buf[2],
        alpha: buf[3],
    }))
}

fn pad_cursor(cursor: &mut io::Cursor<&[u8]>) {
    let pos = cursor.position();
    cursor.set_position(encoder::pad(pos));
}
