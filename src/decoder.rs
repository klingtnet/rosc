use crate::encoder;
use crate::errors::OscError;
use crate::types::{
    OscArray, OscBundle, OscColor, OscMessage, OscMidiMessage, OscPacket, OscType, Result,
};

use std::io::{BufRead, Read};
use std::{char, io};

use byteorder::{BigEndian, ReadBytesExt};

/// Common MTU size for ethernet
pub const MTU: usize = 1536;

/// Takes an byte slice as argument and returns an
/// OSC packet on success or an `OscError` if the slice
/// does not contain a valid OSC message.
pub fn decode(msg: &[u8]) -> Result<OscPacket> {
    if msg.is_empty() {
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

    let addr: String = read_osc_string(&mut cursor)?;
    let type_tags: String = read_osc_string(&mut cursor)?;

    if type_tags.len() > 1 {
        let args: Vec<OscType> = read_osc_args(&mut cursor, type_tags)?;
        Ok(OscPacket::Message(OscMessage { addr, args }))
    } else {
        Ok(OscPacket::Message(OscMessage { addr, args: vec![] }))
    }
}

fn decode_bundle(msg: &[u8]) -> Result<OscPacket> {
    let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(msg);

    let bundle_tag = read_osc_string(&mut cursor)?;
    if bundle_tag != "#bundle" {
        return Err(OscError::BadBundle(format!(
            "Wrong bundle specifier: {}",
            bundle_tag
        )));
    }

    let time_tag = read_time_tag(&mut cursor)?;

    let mut bundle: Vec<OscPacket> = Vec::new();

    let mut elem_size = read_bundle_element_size(&mut cursor)?;

    while msg.len() >= (cursor.position() as usize) + elem_size {
        let packet = read_bundle_element(&mut cursor, elem_size)?;
        bundle.push(packet);

        if msg.len() == cursor.position() as usize {
            break;
        }

        elem_size = read_bundle_element_size(&mut cursor)?;
    }

    Ok(OscPacket::Bundle(OscBundle {
        timetag: time_tag,
        content: bundle,
    }))
}

fn read_bundle_element_size(cursor: &mut io::Cursor<&[u8]>) -> Result<usize> {
    cursor
        .read_u32::<BigEndian>()
        .map(|size| size as usize)
        .map_err(OscError::ReadError)
}

fn read_bundle_element(cursor: &mut io::Cursor<&[u8]>, elem_size: usize) -> Result<OscPacket> {
    let mut buf: Vec<u8> = Vec::with_capacity(elem_size);

    let mut handle = cursor.take(elem_size as u64);

    let cnt = handle.read_to_end(&mut buf).map_err(OscError::ReadError)?;

    if cnt == elem_size {
        decode(&buf)
    } else {
        Err(OscError::BadBundle(
            "Bundle shorter than expected!".to_string(),
        ))
    }
}

fn read_osc_string(cursor: &mut io::Cursor<&[u8]>) -> Result<String> {
    let mut str_buf: Vec<u8> = Vec::new();
    // ignore returned byte count
    cursor
        .read_until(0, &mut str_buf)
        .map_err(OscError::ReadError)?;
    pad_cursor(cursor);
    // convert to String and remove nul bytes
    String::from_utf8(str_buf)
        .map_err(OscError::StringError)
        .map(|s| s.trim_matches(0u8 as char).to_string())
}

fn read_osc_args(cursor: &mut io::Cursor<&[u8]>, raw_type_tags: String) -> Result<Vec<OscType>> {
    let type_tags: Vec<char> = raw_type_tags.chars().skip(1).collect();

    let mut args: Vec<OscType> = Vec::with_capacity(type_tags.len());
    let mut stack: Vec<Vec<OscType>> = Vec::new();
    for tag in type_tags {
        if tag == '[' {
            // array start: save current frame and start a new frame
            // for the array's content
            stack.push(args);
            args = Vec::new();
        } else if tag == ']' {
            // found the end of the current array:
            // create array object from current frame and step one level up
            let array = OscType::Array(OscArray { content: args });
            match stack.pop() {
                Some(stashed) => args = stashed,
                None => return Err(OscError::BadMessage("Encountered ] outside array")),
            }
            args.push(array);
        } else {
            let arg: OscType = read_osc_arg(cursor, tag)?;
            args.push(arg);
        }
    }
    Ok(args)
}

fn read_osc_arg(cursor: &mut io::Cursor<&[u8]>, tag: char) -> Result<OscType> {
    match tag {
        'f' => cursor
            .read_f32::<BigEndian>()
            .map(OscType::Float)
            .map_err(OscError::ReadError),
        'd' => cursor
            .read_f64::<BigEndian>()
            .map(OscType::Double)
            .map_err(OscError::ReadError),
        'i' => cursor
            .read_i32::<BigEndian>()
            .map(OscType::Int)
            .map_err(OscError::ReadError),
        'h' => cursor
            .read_i64::<BigEndian>()
            .map(OscType::Long)
            .map_err(OscError::ReadError),
        's' => read_osc_string(cursor).map(OscType::String),
        't' => read_time_tag(cursor).map(OscType::Time),
        'b' => read_blob(cursor),
        'r' => read_osc_color(cursor),
        'T' => Ok(true.into()),
        'F' => Ok(false.into()),
        'N' => Ok(OscType::Nil),
        'I' => Ok(OscType::Inf),
        'c' => read_char(cursor),
        'm' => read_midi_message(cursor),
        _ => Err(OscError::BadArg(format!(
            "Type tag \"{}\" is not implemented!",
            tag
        ))),
    }
}

fn read_char(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let opt_char = cursor
        .read_u32::<BigEndian>()
        .map(char::from_u32)
        .map_err(OscError::ReadError)?;
    match opt_char {
        Some(c) => Ok(OscType::Char(c)),
        None => Err(OscError::BadArg("Argument is not a char!".to_string())),
    }
}

fn read_blob(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let size: usize = cursor
        .read_u32::<BigEndian>()
        .map_err(OscError::ReadError)? as usize;
    let mut byte_buf: Vec<u8> = Vec::with_capacity(size);

    cursor
        .take(size as u64)
        .read_to_end(&mut byte_buf)
        .map_err(OscError::ReadError)?;

    pad_cursor(cursor);

    Ok(OscType::Blob(byte_buf))
}

fn read_time_tag(cursor: &mut io::Cursor<&[u8]>) -> Result<(u32, u32)> {
    let date = cursor
        .read_u32::<BigEndian>()
        .map_err(OscError::ReadError)?;
    let frac = cursor
        .read_u32::<BigEndian>()
        .map_err(OscError::ReadError)?;

    Ok((date, frac))
}

fn read_midi_message(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let mut buf: Vec<u8> = Vec::with_capacity(4);
    cursor
        .take(4)
        .read_to_end(&mut buf)
        .map_err(OscError::ReadError)?;

    Ok(OscType::Midi(OscMidiMessage {
        port: buf[0],
        status: buf[1],
        data1: buf[2],
        data2: buf[3],
    }))
}

fn read_osc_color(cursor: &mut io::Cursor<&[u8]>) -> Result<OscType> {
    let mut buf: Vec<u8> = Vec::with_capacity(4);
    cursor
        .take(4)
        .read_to_end(&mut buf)
        .map_err(OscError::ReadError)?;

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
