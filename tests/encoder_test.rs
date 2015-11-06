extern crate rosc;

use rosc::{encoder, decoder, errors};
use rosc::errors::OscError;
use rosc::types::{Result, OscMessage, OscPacket};

#[test]
fn test_encode_message() {
    let msg = OscMessage{
        addr: "/some/addr".to_string(),
        args: None,
    };

    let enc_msg = encoder::encode_packet(OscPacket::Message(msg)).unwrap();
    match decoder::decode(&enc_msg).unwrap() {
        OscPacket::Message(dec_msg) => assert_eq!("/some/addr", dec_msg.addr),
        _ => panic!("Expected OSC message!")
    }
}
