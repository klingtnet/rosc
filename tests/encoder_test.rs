extern crate rosc;

use rosc::{decoder, encoder};
use rosc::{OscArray, OscBundle, OscColor, OscMessage, OscMidiMessage, OscPacket, OscType};

#[test]
fn test_encode_message_wo_args() {
    let msg_packet = OscPacket::Message(OscMessage {
        addr: "/some/addr".to_string(),
        args: vec![],
    });

    let enc_msg = encoder::encode(&msg_packet).unwrap();
    assert_eq!(enc_msg.len() % 4, 0);

    let msg = match msg_packet {
        OscPacket::Message(ref msg) => msg,
        _ => panic!(),
    };

    let dec_msg = match decoder::decode_udp(&enc_msg).unwrap().1 {
        OscPacket::Message(m) => m,
        _ => panic!("Expected OscMessage!"),
    };

    assert_eq!(*msg, dec_msg)
}

#[test]
fn test_encode_empty_bundle() {
    let bundle_packet = OscPacket::Bundle(OscBundle {
        timetag: (4, 2).into(),
        content: vec![],
    });

    let enc_bundle = encoder::encode(&bundle_packet).unwrap();
    assert_eq!(enc_bundle.len() % 4, 0);
    assert_eq!(enc_bundle.len(), 16);

    let dec_bundle = match decoder::decode_udp(&enc_bundle).unwrap().1 {
        OscPacket::Bundle(m) => m,
        _ => panic!("Expected OscBundle!"),
    };

    let bundle = match bundle_packet {
        OscPacket::Bundle(ref bundle) => bundle,
        _ => panic!(),
    };

    assert_eq!(*bundle, dec_bundle)
}

#[test]
fn test_encode_message_with_args() {
    let msg_packet = OscPacket::Message(OscMessage {
        addr: "/another/address/1".to_string(),
        args: vec![
            4i32.into(),
            42i64.into(),
            3.1415926f32.into(),
            3.14159265359f64.into(),
            "This is a string.".to_string().into(),
            "This is a string too.".into(),
            vec![1u8, 2u8, 3u8].into(),
            (123, 456).into(),
            'c'.into(),
            false.into(),
            true.into(),
            OscType::Nil,
            OscType::Inf,
            OscMidiMessage {
                port: 4,
                status: 41,
                data1: 42,
                data2: 129,
            }
            .into(),
            OscColor {
                red: 255,
                green: 192,
                blue: 42,
                alpha: 13,
            }
            .into(),
            OscArray {
                content: vec![
                    42i32.into(),
                    OscArray {
                        content: vec![1.23.into(), 3.21.into()],
                    }
                    .into(),
                    "Yay".into(),
                ],
            }
            .into(),
        ],
    });

    let enc_msg = encoder::encode(&msg_packet).unwrap();
    assert_eq!(enc_msg.len() % 4, 0);

    let dec_msg: OscMessage = match decoder::decode_udp(&enc_msg).unwrap().1 {
        OscPacket::Message(m) => m,
        _ => panic!("Expected OscMessage!"),
    };

    let msg = match msg_packet {
        OscPacket::Message(ref msg) => msg,
        _ => panic!(),
    };

    assert_eq!(*msg, dec_msg);
}

#[test]
fn test_encode_bundle() {
    let msg0 = OscMessage {
        addr: "/view/1".to_string(),
        args: vec![],
    };

    let msg1 = OscMessage {
        addr: "/mixer/channel/1/amp".to_string(),
        args: vec![0.9f32.into()],
    };

    let msg2 = OscMessage {
        addr: "/osc/1/freq".to_string(),
        args: vec![440i32.into()],
    };

    let msg3 = OscMessage {
        addr: "/osc/1/phase".to_string(),
        args: vec![(-0.4f32).into()],
    };

    let bundle1 = OscBundle {
        timetag: (5678, 8765).into(),
        content: vec![OscPacket::Message(msg2), OscPacket::Message(msg3)],
    };

    let root_bundle = OscPacket::Bundle(OscBundle {
        timetag: (1234, 4321).into(),
        content: vec![
            OscPacket::Message(msg0),
            OscPacket::Message(msg1),
            OscPacket::Bundle(bundle1),
        ],
    });

    let enc_bundle = encoder::encode(&root_bundle).unwrap();
    assert_eq!(enc_bundle.len() % 4, 0);

    let dec_bundle = decoder::decode_udp(&enc_bundle).unwrap().1;
    assert_eq!(root_bundle, dec_bundle);
}

#[cfg(feature = "std")]
#[test]
fn test_encode_bundle_into_cursor() {
    let msg0 = OscMessage {
        addr: "/view/1".to_string(),
        args: vec![],
    };

    let msg1 = OscMessage {
        addr: "/mixer/channel/1/amp".to_string(),
        args: vec![0.9f32.into()],
    };

    let msg2 = OscMessage {
        addr: "/osc/1/freq".to_string(),
        args: vec![440i32.into()],
    };

    let msg3 = OscMessage {
        addr: "/osc/1/phase".to_string(),
        args: vec![(-0.4f32).into()],
    };

    let bundle1 = OscBundle {
        timetag: (5678, 8765).into(),
        content: vec![OscPacket::Message(msg2), OscPacket::Message(msg3)],
    };

    let root_bundle = OscPacket::Bundle(OscBundle {
        timetag: (1234, 4321).into(),
        content: vec![
            OscPacket::Message(msg0),
            OscPacket::Message(msg1),
            OscPacket::Bundle(bundle1),
        ],
    });

    let mut buffer = Vec::new();
    encoder::encode_into(
        &root_bundle,
        &mut encoder::WriteOutput(std::io::Cursor::new(&mut buffer)),
    )
    .unwrap();
    assert_eq!(buffer.len() % 4, 0);

    let dec_bundle = decoder::decode_udp(&buffer).unwrap().1;
    assert_eq!(root_bundle, dec_bundle);
}
