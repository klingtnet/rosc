extern crate rosc;

use rosc::{encoder, decoder};
use rosc::types::{OscMessage, OscMidiMessage, OscColor, OscPacket, OscType};

#[test]
fn test_encode_message_wo_args() {
    let msg = OscMessage {
        addr: "/some/addr".to_string(),
        args: None,
    };

    let enc_msg = encoder::encode_message(&msg).unwrap();
    assert_eq!(enc_msg.len() % 4, 0);

    match decoder::decode(&enc_msg).unwrap() {
        OscPacket::Message(dec_msg) => assert_eq!(msg.addr, dec_msg.addr),
        _ => panic!("Expected OSC message!"),
    }
}

#[test]
fn test_encode_message_with_args() {
    let addr = "/another/address/1";
    let string_arg = "This is a string";
    let blob_arg = vec![1u8, 2u8, 3u8];
    let time_arg = (123, 456);
    let int_arg = 4;
    let float_arg = 3.14;
    let long_arg = 42;
    let double_arg = 3.1415;
    let char_arg = 'c';
    let midi_arg = OscMidiMessage {
        port: 12,
        status: 14,
        data1: 120,
        data2: 53,
    };
    let color_arg = OscColor {
        red: 255u8,
        green: 192u8,
        blue: 42u8,
        alpha: 13u8,
    };
    let args = vec![OscType::Int(int_arg),
                    OscType::Float(float_arg),
                    OscType::String(string_arg.to_string()),
                    OscType::Blob(blob_arg.iter().cloned().collect()),
                    OscType::Time(time_arg.0, time_arg.1),
                    OscType::Long(long_arg),
                    OscType::Double(double_arg),
                    OscType::Char(char_arg),
                    OscType::Bool(false),
                    OscType::Bool(true),
                    OscType::Nil,
                    OscType::Inf,
                    OscType::Midi(OscMidiMessage {
                        port: midi_arg.port,
                        status: midi_arg.status,
                        data1: midi_arg.data1,
                        data2: midi_arg.data2,
                    })];
    let arg_cnt = args.len();

    let msg = OscMessage {
        addr: addr.to_string(),
        args: Some(args),
    };

    let enc_msg = encoder::encode_message(&msg).unwrap();
    assert_eq!(enc_msg.len() % 4, 0);

    let dec_msg: OscMessage = match decoder::decode(&enc_msg).unwrap() {
        OscPacket::Message(m) => m,
        _ => panic!("Expected OscMessage!"),
    };

    // check if osc address is equal
    assert_eq!(addr, dec_msg.addr);
    // check if there are arguments
    assert!(dec_msg.args.is_some());
    let dec_args = dec_msg.args.unwrap();

    // check if argument count is equal
    assert_eq!(arg_cnt, dec_args.len());

    for arg in dec_args {
        match arg {
            OscType::Int(x) => assert_eq!(int_arg, x),
            OscType::Long(x) => assert_eq!(long_arg, x),
            OscType::Float(x) => assert_eq!(float_arg, x),
            OscType::Double(x) => assert_eq!(double_arg, x),
            OscType::String(x) => assert_eq!(string_arg, x),
            OscType::Blob(x) => assert_eq!(blob_arg, x),
            OscType::Time(x, y) => assert_eq!(time_arg, (x, y)),
            OscType::Color(x) => {
                assert_eq!(color_arg.red, x.red);
                assert_eq!(color_arg.green, x.green);
                assert_eq!(color_arg.blue, x.blue);
                assert_eq!(color_arg.alpha, x.alpha);
            }
            OscType::Char(x) => assert_eq!(char_arg, x),
            OscType::Bool(_) => (),
            OscType::Inf => (),
            OscType::Nil => (),
            OscType::Midi(x) => {
                assert_eq!(midi_arg.port, x.port);
                assert_eq!(midi_arg.status, x.status);
                assert_eq!(midi_arg.data1, x.data1);
                assert_eq!(midi_arg.data2, x.data2);
            }
            _ => panic!("Unexpected OSC argument!"),
        }
    }
}
