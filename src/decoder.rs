use types::{OscPacket, OscType, OscResult, OscMessage, OscMidiMessage, OscBundle};
use errors::OscError;
use utils;

use std::{io, char};
use std::io::{Read, BufRead};

use byteorder::{BigEndian, ReadBytesExt};

/// Common MTP size for ethernet
pub const MTP: usize = 1536;

pub fn decode(msg: &[u8]) -> OscResult<OscPacket> {
    match msg[0] as char {
        '/' => {
            decode_message(msg)
        }
        '#' => {
            decode_bundle(msg)
        }
        _ => Err(OscError::BadPacket("Unknown message format.".to_string())),
    }
}

fn decode_message(msg: &[u8]) -> OscResult<OscPacket> {
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

fn decode_bundle(msg: &[u8]) -> OscResult<OscPacket> {
    let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(msg);

    let b = try!(read_osc_string(&mut cursor));
    if b != "bundle" {
        return Err(OscError::BadBundle);
    }

    let time_tag = try!(read_time_tag(&mut cursor));
    let mut bundle: Vec<OscPacket> = Vec::new();
    let size: usize = try!(cursor.read_u32::<BigEndian>()
                                 .map_err(OscError::ByteOrderError)) as usize;
    let mut buf: Vec<u8> = Vec::new();
    let cnt: usize = try!(cursor.take(size as u64)
                                .read_to_end(&mut buf)
                                .map_err(OscError::ReadError));
    if cnt == size {
        try!(decode(&mut buf).map(|p| bundle.push(p)));
    } else {
        return Err(OscError::BadBundle);
    }

    Ok(OscPacket::Bundle(OscBundle {
        timetag: time_tag,
        content: bundle,
    }))
}

fn read_osc_string(cursor: &mut io::Cursor<&[u8]>) -> OscResult<String> {
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

fn read_osc_args(cursor: &mut io::Cursor<&[u8]>, raw_type_tags: String) -> OscResult<Vec<OscType>> {
    let type_tags: Vec<char> = raw_type_tags.chars()
                                            .skip(1)
                                            .map(|c| c as char)
                                            .collect();
    let mut args: Vec<OscType> = Vec::with_capacity(type_tags.len());
    for tag in type_tags {
        let arg: OscType = try!(read_osc_arg(cursor, tag));
        args.push(arg);
    }
    Ok(args)
}

fn read_osc_arg(cursor: &mut io::Cursor<&[u8]>, tag: char) -> OscResult<OscType> {
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
        's' => {
            read_osc_string(cursor).map(|s| OscType::String(s))
        }
        't' => {
            read_time_tag(cursor)
        }
        'b' => {
            read_blob(cursor)
        }
        'T' => {
            Ok(OscType::Bool(true))
        }
        'F' => {
            Ok(OscType::Bool(false))
        }
        'N' => {
            Ok(OscType::Nil)
        }
        'I' => {
            Ok(OscType::Inf)
        }
        'c' => {
            let opt_char = try!(cursor.read_u32::<BigEndian>()
                                      .map(char::from_u32)
                                      .map_err(OscError::ByteOrderError));
            match opt_char {
                Some(c) => Ok(OscType::Char(c)),
                None => Err(OscError::BadArg("Argument is not a char!".to_string())),
            }
        }
        'm' => {
            read_midi_message(cursor)
        }
        _ => Err(OscError::BadArg(format!("Type tag \"{}\" is not implemented!", tag))),
    }
}

fn read_blob(cursor: &mut io::Cursor<&[u8]>) -> OscResult<OscType> {
    let size: usize = try!(cursor.read_u32::<BigEndian>()
                                 .map_err(OscError::ByteOrderError)) as usize;
    let mut byte_buf: Vec<u8> = Vec::with_capacity(size);

    try!(cursor.take(size as u64)
               .read_to_end(&mut byte_buf)
               .map_err(OscError::ReadError));

    pad_cursor(cursor);

    Ok(OscType::Blob(byte_buf))
}

fn read_time_tag(cursor: &mut io::Cursor<&[u8]>) -> OscResult<OscType> {
    let date = try!(cursor.read_u32::<BigEndian>()
                          .map_err(OscError::ByteOrderError));
    let frac = try!(cursor.read_u32::<BigEndian>()
                          .map_err(OscError::ByteOrderError));

    Ok(OscType::Time(date, frac))
}

fn read_midi_message(cursor: &mut io::Cursor<&[u8]>) -> OscResult<OscType> {
    let mut buf: Vec<u8> = Vec::with_capacity(4);
    try!(cursor.take(4).read_to_end(&mut buf).map_err(OscError::ReadError));

    Ok(OscType::Midi(OscMidiMessage {
        port: buf[0],
        status: buf[1],
        data1: buf[2],
        data2: buf[3],
    }))

}

fn pad_cursor(cursor: &mut io::Cursor<&[u8]>) {
    let pos = cursor.position();
    cursor.set_position(utils::pad(pos));
}
