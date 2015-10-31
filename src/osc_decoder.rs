use {osc_types as ot,
    errors as oe};

use std::{io, string, mem, error};
use std::io::BufRead;

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
            println!("{}",try!(read_osc_string(&mut cursor)));
        }
        Err(e) => {
            return Err(e)
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