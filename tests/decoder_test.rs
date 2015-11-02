extern crate rosc;
extern crate byteorder;

use byteorder::{ByteOrder, BigEndian};
use std::{mem, iter};
use std::ascii::AsciiExt;

use rosc::{types, errors, decoder, utils};

#[test]
fn test_decode_no_args() {
    // message to build: /some/valid/address/4 ,
    let raw_addr = "/some/valid/address/4";
    let addr = to_osc_string(raw_addr.as_bytes());
    let type_tags = to_osc_string(b",");
    let merged: Vec<u8> = addr.into_iter()
                              .chain(type_tags.into_iter())
                              .collect();
    let osc_packet: Result<types::OscPacket, errors::OscError> = decoder::decode(&merged);
    assert!(osc_packet.is_ok());
    match osc_packet {
        Ok(types::OscPacket::Message(msg)) => {
            assert_eq!(raw_addr, msg.addr);
            assert!(msg.args.is_none());
        }
        Ok(_) => panic!("Expected an OscMessage!"),
        Err(e) => panic!(e),
    }
}

#[test]
fn test_decode_args() {
    // /another/valid/address/123 ,fdih 3.1415 3.14159265359 12345678i32
    // -1234567891011
    let raw_addr = "/another/valid/address/123";
    let addr = to_osc_string(raw_addr.as_bytes());
    // args
    let f = 3.1415f32;
    let mut f_bytes: [u8; 4] = [0u8; 4];
    BigEndian::write_f32(&mut f_bytes, f);
    assert_eq!(BigEndian::read_f32(&f_bytes), f);

    let d = 3.14159265359f64;
    let mut d_bytes: [u8; 8] = [0u8; 8];
    BigEndian::write_f64(&mut d_bytes, d);
    assert_eq!(BigEndian::read_f64(&d_bytes), d);

    let i = 12345678i32;
    let i_bytes: [u8; 4] = unsafe { mem::transmute(i.to_be()) };

    let l = -1234567891011i64;
    let h_bytes: [u8; 8] = unsafe { mem::transmute(l.to_be()) };

    let s = "I am an osc test string.";
    assert!(s.is_ascii());

    // Osc strings are null terminated like in C!
    let s_bytes: Vec<u8> = to_osc_string(s.as_bytes());

    let type_tags = to_osc_string(b",fdsih");

    let args: Vec<u8> = f_bytes.iter()
                               .chain(d_bytes.iter())
                               .chain(s_bytes.iter())
                               .chain(i_bytes.iter())
                               .chain(h_bytes.iter())
                               .map(|x| *x)
                               .collect::<Vec<u8>>();

    let merged: Vec<u8> = addr.into_iter()
                              .chain(type_tags.into_iter())
                              .chain(args)
                              .collect::<Vec<u8>>();

    let osc_packet: Result<types::OscPacket, errors::OscError> = decoder::decode(&merged);
    match osc_packet {
        Ok(types::OscPacket::Message(msg)) => {
            assert_eq!(raw_addr, msg.addr);
            for arg in msg.args.unwrap() {
                match arg {
                    types::OscType::Int(x) => assert_eq!(i, x),
                    types::OscType::Long(x) => assert_eq!(l, x),
                    types::OscType::Float(x) => assert_eq!(f, x),
                    types::OscType::Double(x) => assert_eq!(d, x),
                    types::OscType::String(x) => assert_eq!(s, x),
                    _ => panic!(),
                }
            }
        }
        Ok(_) => panic!("Expected an OscMessage!"),
        Err(e) => panic!(e),
    }
}

/// Nul terminates the data and pads with zero bytes to a length that is a multiple of 4.
fn to_osc_string(data: &[u8]) -> Vec<u8> {
    let mut v: Vec<u8> = data.iter()
                             .cloned()
                             .chain(iter::once(0u8))
                             .collect();
    // v;
    let pad_len: usize = utils::pad(v.len() as u64) as usize;
    while v.len() < pad_len {
        v.push(0u8);
    }
    v
}