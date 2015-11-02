extern crate rosc;

use rosc::{types, errors, decoder, utils};

#[test]
fn test_decode_no_args() {
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
        Ok(packet) => match packet {
            types::OscPacket::Message(msg) => {
                assert_eq!(raw_addr, msg.addr);
                assert!(msg.args.is_none());
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