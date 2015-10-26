use {osc_types, errors};

use std::{io, str, mem};

use byteorder;
use byteorder::ReadBytesExt;

/// Common MTP size for ethernet
pub const MTP: usize = 1536;

pub fn decode(msg: &[u8], size: usize) -> Result<osc_types::OscPacket, errors::OscError> {
    match msg[0] as char {
        '/' => {
            decode_message(msg, size);
            Ok(osc_types::OscPacket::Message(osc_types::OscMessage { addr: "".to_string() }))
        }
        '#' => {
            decode_bundle(msg);
            Ok(osc_types::OscPacket::Bundle)
        }
        _ => Err(errors::OscError::BadOscPacket("Unknown message format.".to_string())),
    }
}

fn decode_message(msg: &[u8], size: usize) -> Result<osc_types::OscPacket, errors::OscError> {
    let osc_msg: osc_types::OscMessage = osc_types::OscMessage { addr: "".to_string() };
    const zero: u8 = 0u8;
    let mut pos: usize = 0;

    let addr: &str;
    let args: Vec<osc_types::OscType>;

    // implement method that returns zero terminated 32-bit aligned blocks
    match decode_addr(&msg) {
        Ok((s, end)) => {
            addr = s;
            pos = end;
        }
        Err(e) => return Err(e),
    }

    // decode types directly
    match decode_type_tags(&msg[pos..]) {
        Ok((tags, end)) => {
            pos += end;
            match tags {
                Some(t) => {
                    decode_args(&msg[pos..], t);
                    ()
                }
                None => {
                    // address only messages, like view switches
                    return Ok(osc_types::OscPacket::Message(osc_msg));
                }
            }
        }
        Err(e) => return Err(e),
    }

    Ok(osc_types::OscPacket::Message(osc_msg))
}

fn decode_bundle(msg: &[u8]) -> Result<osc_types::OscPacket, errors::OscError> {
    Err(errors::OscError::BadOscBundle)
}

fn decode_addr(msg: &[u8]) -> Result<(&str, usize), errors::OscError> {
    match decode_osc_string(&msg) {
        Ok((s, pos)) => Ok((s, pos)),
        Err(e) => Err(errors::OscError::BadOscAddress(format!("Bad address: {}", e))),
    }
}

// can be empty, return Option
fn decode_type_tags(msg: &[u8]) -> Result<(Option<Vec<char>>, usize), errors::OscError> {
    match decode_osc_string(&msg) {
        Ok((s, pos)) => {
            if s.len() == 1 {
                Ok((None, pos))
            } else {
                let tags: Vec<char> = s.chars()
                                       .skip(1)
                                       .map(|c| c as char)
                                       .collect();
                Ok((Some(tags), pos))
            }
        }
        Err(e) => Err(errors::OscError::BadOscAddress(format!("Bad type tags: {}", e))),
    }
}

fn decode_args(msg: &[u8], tags: Vec<char>) -> Result<Vec<osc_types::OscType>, errors::OscError> {
    let mut args: Vec<osc_types::OscType> = Vec::with_capacity(tags.len());
    let mut pos: usize = 0;
    for tag in tags {
        match decode_arg(&msg[pos..], tag) {
            Ok((arg, end)) => {
                args.push(arg);
                pos+=end;
            },
            Err(e) => {
                return Err(e)
            }
        }
    }
    Ok(args)
}

fn decode_arg(msg: &[u8], tag: char) -> Result<(osc_types::OscType, usize), errors::OscError> {
    match tag {
        'i' => {
                let mut buf = io::Cursor::new(msg);
                let num = buf.read_u32::<byteorder::BigEndian>().unwrap();

                // let mut buf: [u8; 4] = [0u8; 4];
                // buf.clone_from_slice(&msg[0..4]);
                // let i: i32 = mem::transmute::<[u8; 4], i32>(buf);
                println!("{}", num)
        },
        'f' => {
                let mut buf = io::Cursor::new(msg);
                let f = buf.read_f32::<byteorder::BigEndian>().unwrap();
                // let mut buf: [u8; 4] = [0u8; 4];
                // buf.clone_from_slice(&msg[0..4]);
                // let f: f32 = mem::transmute::<[u8; 4], f32>(buf);
                println!("{}", f)
            },
        's' => (),
        _ => (),
    };
    Ok((osc_types::OscType::OscInt(3i32), 4usize))
}

fn decode_osc_string(msg: &[u8]) -> Result<(&str, usize), errors::OscError> {
    match msg.iter().position(|x| *x == 0) {
        Some(pos) => {
            let aligned_end: usize = pad_four(pos);
            if aligned_end >= msg.len() {
                Err(errors::OscError::BadOscString("String not 32-bit aligned!".to_string()))
            } else {
                match as_utf8(&msg[0..aligned_end]) {
                    Ok(s) => Ok((s, aligned_end)),
                    Err(e) => Err(errors::OscError::BadOscString(format!("{}", e))),
                }
            }
        }
        None => Err(errors::OscError::BadOscString("OSC string not null terminated!".to_string())),
    }
}

fn as_utf8(msg: &[u8]) -> Result<&str, str::Utf8Error> {
    match str::from_utf8(&msg) {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}

fn pad_four(pos: usize) -> usize {
    let d: usize = pos % 4;
    match d {
        0 => pos,
        _ => pos + (4 - d),
    }
}
