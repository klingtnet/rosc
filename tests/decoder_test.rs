extern crate rosc;
extern crate byteorder;

use byteorder::{ByteOrder, BigEndian};
use std::mem;
use rosc::{types, errors, decoder, utils};

#[test]
fn test_decode_no_args() {
    // message to build: /some/valid/address/4 ,
    let raw_addr = "/some/valid/address/4";
    let addr = pad(raw_addr.as_bytes());
    // args
    let type_tags = pad(b",");
    let merged: Vec<u8> = addr.into_iter()
        .chain(type_tags.into_iter())
        .collect();
    let osc_packet: Result<types::OscPacket, errors::OscError> = decoder::decode(&merged);
    assert!(osc_packet.is_ok());
    match osc_packet {
        Ok(types::OscPacket::Message(msg)) => {
                assert_eq!(raw_addr, msg.addr);
                assert!(msg.args.is_none());
        },
        Ok(_) => panic!("OscMessage was expected!"),
        Err(e) => panic!(e)
    }
}

#[test]
fn test_decode_args() {
    // /another/valid/address/123 ,fdih 3.1415 3.14159265359 12345678i32
    // -1234567891011
    let raw_addr = "/another/valid/address/123";
    let addr = pad(raw_addr.as_bytes());
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
    let i_bytes: [u8; 4] = unsafe {mem::transmute(i.to_be())};

    let l = -1234567891011i64;
    let h_bytes: [u8; 8] = unsafe {mem::transmute(l.to_be())};

    let type_tags = pad(b",fdih");

    let args: Vec<u8> = f_bytes.iter()
        .chain(d_bytes.iter())
        .chain(i_bytes.iter())
        .chain(h_bytes.iter())
        .map(|x| *x)
        .collect::<Vec<u8>>();

    let merged: Vec<u8> = addr.into_iter()
        .chain(type_tags.into_iter())
        .chain(args)
        .collect::<Vec<u8>>();
    let osc_packet: Result<types::OscPacket, errors::OscError> = decoder::decode(&merged);
    assert!(osc_packet.is_ok());
    match osc_packet {
        Ok(packet) => match packet {
            types::OscPacket::Message(msg) => {
                assert_eq!(raw_addr, msg.addr);
                match msg.args {
                    Some(args) => {
                        for arg in args {
                            match arg {
                                types::OscType::OscInt(x) => assert_eq!(i,x),
                                types::OscType::OscLong(x) => assert_eq!(l,x),
                                types::OscType::OscFloat(x) => assert_eq!(f, x),
                                types::OscType::OscDouble(x) => assert_eq!(d, x),
                                _ => panic!()
                            }
                        }
                    },
                    None => panic!()
                }
            },
            _ => panic!()
        },
        Err(e) => panic!(e)
    }
}

fn pad(data: &[u8]) -> Vec<u8> {
    let pad_len: usize = utils::pad(data.len() as u64) as usize;
    let mut v: Vec<u8> = data.iter().cloned().collect();
    while v.len() < pad_len {
        v.push(0u8);
    }
    v
}