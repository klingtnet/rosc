use {osc_types, errors};

use std::{io, str};
use std::io::BufRead; // trait

/// Common MTP size for ethernet
pub const MTP: usize = 1536;

pub fn destruct(msg: &[u8], size: usize) -> Result<osc_types::OscPacket, errors::OscError> {
    match msg[0] as char {
        '/' => {
            destruct_message(msg, size);
            Ok(osc_types::OscPacket::OscMessage)
        }
        '#' => {
            destruct_bundle(msg);
            Ok(osc_types::OscPacket::OscBundle)
        }
        _ => Err(errors::OscError::BadOscPacket("Unknown message format.".to_string())),
    }
}

fn destruct_message(msg: &[u8], size: usize) -> Result<osc_types::OscPacket, errors::OscError> {
    let buffer: &mut Vec<u8> = &mut Vec::new();
    let mut reader = io::BufReader::with_capacity(size, msg);
    let mut pos: usize = 0;
    match reader.read_until(0u8, buffer) {
        Ok(pos) => {
            match read_as_utf8(&msg[0..pos]) {
                Ok(s) => {
                    println!("address: {}, pad: {}", s, pad_four(pos));
                    Ok(osc_types::OscPacket::OscMessage)
                },
                Err(e) => Err(errors::OscError::BadOscAddress("Could not interpret address.".to_string()))
            }
        }
        Err(e) => Err(errors::OscError::BadOscPacket("Broken message.".to_string())),
    }
}

fn destruct_bundle(msg: &[u8]) -> Result<osc_types::OscPacket, errors::OscError> {
    Err(errors::OscError::BadOscBundle)
}

fn read_as_utf8(msg: &[u8]) -> Result<&str, str::Utf8Error> {
    match str::from_utf8(&msg) {
        Ok(s) => Ok(s),
        Err(e) => Err(e)
    }
}

fn pad_four(pos: usize) -> usize {
    let d: usize = pos % 4;
    match d {
        0 => pos,
        _ => pos + (4 - d)
    }
}