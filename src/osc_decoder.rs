use {osc_types as ot,
    errors as oe};

use std::{io, string, mem, error};
use std::io::{Read, BufRead};

use byteorder;
use byteorder::{BigEndian, ReadBytesExt};

/// Common MTP size for ethernet
pub const MTP: usize = 1536;

pub fn decode(msg: &[u8], size: usize) -> ot::OscResult<ot::OscPacket> {
    match msg[0] as char {
        '/' => {
            decode_message(msg, size)
        }
        '#' => {
            decode_bundle(msg)
        }
        _ => Err(oe::OscError::BadOscPacket("Unknown message format.".to_string())),
    }
}

fn decode_message(msg: &[u8], size: usize) -> ot::OscResult<ot::OscPacket> {
    let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(msg);
    let mut pos: u64 = 0;

    match read_osc_string(&mut cursor) {
        Ok(s) => {
            let addr: String = s;
            match read_osc_string(&mut cursor) {
                Ok(type_tags) => {
                    if type_tags.len() > 1 {
                        match read_osc_args(&mut cursor, type_tags) {
                            Ok(args) => println!("{}", args.len()),
                            Err(e) => ()
                        };
                    }
                },
                Err(e) => return Err(e)
            }
        }
        Err(e) => return Err(e)
    }

    Ok(ot::OscPacket::Message(ot::OscMessage))
}

fn read_osc_string(cursor: &mut io::Cursor<&[u8]>) -> ot::OscResult<String> {
    let mut str_buf: Vec<u8> = Vec::new();
    match cursor.read_until(0, &mut str_buf) {
        Ok(_) => {
            pad_cursor(cursor);
            String::from_utf8(str_buf)
                .map_err(|e| oe::OscError::StringError(e))
                .map(|s| s.trim_matches(0u8 as char).to_string())
        },
        Err(e) => Err(oe::OscError::ReadError(e)),
    }
}

fn read_osc_args(cursor: &mut io::Cursor<&[u8]>, raw_type_tags: String) -> ot::OscResult<Vec<ot::OscType>> {
    let type_tags: Vec<char> = raw_type_tags.chars()
        .skip(1)
        .map(|c| c as char)
        .collect();
    let mut args: Vec<ot::OscType> = Vec::with_capacity(type_tags.len());
    for tag in type_tags {
        match read_osc_arg(cursor, tag) {
            Ok(arg) => {
                args.push(arg);
            },
            Err(e) => return Err(e)
        }
    }
    Ok(args)
}

fn read_osc_arg(cursor: &mut io::Cursor<&[u8]>, tag: char) -> ot::OscResult<ot::OscType> {
    match tag {
        'f' => {
            cursor.read_f32::<BigEndian>()
            .map(|f| ot::OscType::OscFloat(f))
            .map_err(|e| oe::OscError::ByteOrderError(e))
        },
        'd' => {
            cursor.read_f64::<BigEndian>()
            .map(|d| ot::OscType::OscDouble(d))
            .map_err(|e| oe::OscError::ByteOrderError(e))
        }
        'i' => {
            cursor.read_i32::<BigEndian>()
            .map(|i| ot::OscType::OscInt(i))
            .map_err(|e| oe::OscError::ByteOrderError(e))
        },
        'h' => {
            cursor.read_i64::<BigEndian>()
            .map(|l| ot::OscType::OscLong(l))
            .map_err(|e| oe::OscError::ByteOrderError(e))
        },
        's' => {
            read_osc_string(cursor)
            .map(|s| ot::OscType::OscString(s))
        },
        'b' => {
            match cursor.read_u32::<BigEndian>() {
                Ok(size) => {
                    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
                    match cursor.take(size as u64).read(&mut buf) {
                        Ok(blob_size) => Ok(ot::OscType::OscBlob(buf)),
                        Err(e) => Err(oe::OscError::ReadError(e))
                    }
                },
                Err(e) => return Err(oe::OscError::BadOscBundle)
            }
            // int32 byte count followed by as many bytes
            // read blob ...
        },
        _ => Err(oe::OscError::BadOscArg(format!("Type tag \"{}\" is not implemented!", tag))),
    }
}

fn decode_bundle(msg: &[u8]) -> ot::OscResult<ot::OscPacket> {
    Err(oe::OscError::BadOscBundle)
}

fn pad_cursor(cursor: &mut io::Cursor<&[u8]>) {
    let pos = cursor.position();
    match pos % 4 {
        0 => (),
        d => cursor.set_position(pos + (4 - d)),
    }
}