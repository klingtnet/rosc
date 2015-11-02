use types::{OscPacket, OscType, OscResult, OscMessage, OscBundle};
use errors::OscError;
use utils;

use std::io;
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
    match cursor.read_until(0, &mut str_buf) {
        Ok(_) => {
            pad_cursor(cursor);
            String::from_utf8(str_buf)
                .map_err(|e| OscError::StringError(e))
                .map(|s| s.trim_matches(0u8 as char).to_string())
        }
        Err(e) => Err(OscError::ReadError(e)),
    }
}

fn read_osc_args(cursor: &mut io::Cursor<&[u8]>,
                 raw_type_tags: String)
                 -> OscResult<Vec<OscType>> {
    let type_tags: Vec<char> = raw_type_tags.chars()
                                            .skip(1)
                                            .map(|c| c as char)
                                            .collect();
    let mut args: Vec<OscType> = Vec::with_capacity(type_tags.len());
    for tag in type_tags {
        match read_osc_arg(cursor, tag) {
            Ok(arg) => {
                args.push(arg);
            }
            Err(e) => return Err(e),
        }
    }
    Ok(args)
}

fn read_osc_arg(cursor: &mut io::Cursor<&[u8]>, tag: char) -> OscResult<OscType> {
    match tag {
        'f' => {
            cursor.read_f32::<BigEndian>()
                  .map(|f| OscType::Float(f))
                  .map_err(|e| OscError::ByteOrderError(e))
        }
        'd' => {
            cursor.read_f64::<BigEndian>()
                  .map(|d| OscType::Double(d))
                  .map_err(|e| OscError::ByteOrderError(e))
        }
        'i' => {
            cursor.read_i32::<BigEndian>()
                  .map(|i| OscType::Int(i))
                  .map_err(|e| OscError::ByteOrderError(e))
        }
        'h' => {
            cursor.read_i64::<BigEndian>()
                  .map(|l| OscType::Long(l))
                  .map_err(|e| OscError::ByteOrderError(e))
        }
        's' => {
            read_osc_string(cursor).map(|s| OscType::String(s))
        }
        't' => {
            // http://opensoundcontrol.org/node/3/#timetags
            read_time_tag(cursor)
        }
        'b' => {
            match cursor.read_u32::<BigEndian>() {
                Ok(size) => {
                    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
                    match cursor.take(size as u64).read(&mut buf) {
                        Ok(blob_size) => Ok(OscType::Blob(buf)),
                        Err(e) => Err(OscError::ReadError(e)),
                    }
                }
                // TODO: use correct error type
                Err(e) => return Err(OscError::BadBundle),
            }
        }
        _ => Err(OscError::BadArg(format!("Type tag \"{}\" is not implemented!", tag))),
    }
}

fn read_time_tag(cursor: &mut io::Cursor<&[u8]>) -> OscResult<OscType> {
    match cursor.read_u32::<BigEndian>() {
        Ok(date) => {
            match cursor.read_u32::<BigEndian>() {
                Ok(frac) => Ok(OscType::Time(date, frac)),
                Err(e) => Err(OscError::ByteOrderError(e)),
            }
        }
        Err(e) => Err(OscError::ByteOrderError(e)),
    }
}

fn pad_cursor(cursor: &mut io::Cursor<&[u8]>) {
    let pos = cursor.position();
    cursor.set_position(utils::pad(pos));
}
