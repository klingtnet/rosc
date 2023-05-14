extern crate rosc;

use rosc::encoder::pad;
use rosc::{decoder, encoder};
use rosc::{OscArray, OscBundle, OscColor, OscMessage, OscMidiMessage, OscPacket, OscType};

#[test]
fn test_pad() {
    assert_eq!(4, pad(4));
    assert_eq!(8, pad(5));
    assert_eq!(8, pad(6));
    assert_eq!(8, pad(7));
}

#[test]
fn test_encode_message_wo_args() {
    let packet = OscPacket::Message(OscMessage {
        addr: "/some/addr".to_string(),
        args: vec![],
    });

    let bytes = encoder::encode(&packet).unwrap();
    assert_eq!(bytes.len() % 4, 0);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet)
}

#[test]
fn test_encode_empty_bundle() {
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (4, 2).into(),
        content: vec![],
    });

    let bytes = encoder::encode(&packet).unwrap();
    assert_eq!(bytes.len() % 4, 0);
    assert_eq!(bytes.len(), 16);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet)
}

#[test]
fn test_encode_message_with_args() {
    let packet = OscPacket::Message(OscMessage {
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

    let bytes = encoder::encode(&packet).unwrap();
    assert_eq!(bytes.len() % 4, 0);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet)
}

#[test]
fn test_encode_bundle() {
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (1234, 4321).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/view/1".to_string(),
                args: vec![],
            }),
            OscPacket::Message(OscMessage {
                addr: "/mixer/channel/1/amp".to_string(),
                args: vec![0.9f32.into()],
            }),
            OscPacket::Bundle(OscBundle {
                timetag: (5678, 8765).into(),
                content: vec![
                    OscPacket::Message(OscMessage {
                        addr: "/osc/1/freq".to_string(),
                        args: vec![440i32.into()],
                    }),
                    OscPacket::Message(OscMessage {
                        addr: "/osc/1/phase".to_string(),
                        args: vec![(-0.4f32).into()],
                    }),
                ],
            }),
        ],
    });

    let bytes = encoder::encode(&packet).unwrap();
    assert_eq!(bytes.len() % 4, 0);

    let decoded_packet = decoder::decode_udp(&bytes).unwrap().1;
    assert_eq!(packet, decoded_packet);
}

#[cfg(feature = "std")]
#[test]
fn test_encode_bundle_into_cursor() {
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (1234, 4321).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/view/1".to_string(),
                args: vec![],
            }),
            OscPacket::Message(OscMessage {
                addr: "/mixer/channel/1/amp".to_string(),
                args: vec![0.9f32.into()],
            }),
            OscPacket::Bundle(OscBundle {
                timetag: (5678, 8765).into(),
                content: vec![
                    OscPacket::Message(OscMessage {
                        addr: "/osc/1/freq".to_string(),
                        args: vec![440i32.into()],
                    }),
                    OscPacket::Message(OscMessage {
                        addr: "/osc/1/phase".to_string(),
                        args: vec![(-0.4f32).into()],
                    }),
                ],
            }),
        ],
    });

    let mut bytes = Vec::new();
    encoder::encode_into(
        &packet,
        &mut encoder::WriteOutput(std::io::Cursor::new(&mut bytes)),
    )
    .unwrap();
    assert_eq!(bytes.len() % 4, 0);

    let decoded_packet = decoder::decode_udp(&bytes).unwrap().1;
    assert_eq!(packet, decoded_packet);
}
