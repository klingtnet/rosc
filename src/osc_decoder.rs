use {osc_types, errors};

use std::{io, string, mem, error};
use std::io::BufRead;

use byteorder;
use byteorder::{BigEndian, ReadBytesExt};

/// Common MTP size for ethernet
pub const MTP: usize = 1536;

pub fn decode(msg: &[u8], size: usize) -> osc_types::OscResult<osc_types::OscPacket> {
    match msg[0] as char {
        '/' => {
            decode_message(msg, size)
        }
        '#' => {
            decode_bundle(msg)
        }
        _ => Err(errors::OscError::BadOscPacket("Unknown message format.".to_string())),
    }
}

fn decode_message(msg: &[u8], size: usize) -> osc_types::OscResult<osc_types::OscPacket> {
    let mut cursor: io::Cursor<&[u8]> = io::Cursor::new(msg);
    let mut pos: u64 = 0;

    match read_osc_string(&mut cursor) {
        Ok(s) => {
            let addr: String = s;
            println!("{}, {}", addr, pos);
        }
        Err(e) => {
            println!("{}", e)
        }
    }

    Ok(osc_types::OscPacket::Message(osc_types::OscMessage))
}

fn read_osc_string(cursor: &mut io::Cursor<&[u8]>) -> osc_types::OscResult<String> {
    let mut str_buf: Vec<u8> = Vec::new();
    match cursor.read_until(0, &mut str_buf) {
        Ok(_) => {
            pad_cursor(cursor);
            String::from_utf8(str_buf).map_err(|e| errors::OscError::StringError(e))
        },
        Err(e) => Err(errors::OscError::ReadError(e)),
    }
}

fn decode_bundle(msg: &[u8]) -> osc_types::OscResult<osc_types::OscPacket> {
    Err(errors::OscError::BadOscBundle)
}

fn pad_cursor(cursor: &mut io::Cursor<&[u8]>) {
    let pos = cursor.position();
    match pos % 4 {
        0 => (),
        d => cursor.set_position(pos + (4 - d)),
    }
}