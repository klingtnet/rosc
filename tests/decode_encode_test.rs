extern crate rosc;

use rosc::{decoder, encoder};
use rosc::{OscArray, OscBundle, OscColor, OscMessage, OscMidiMessage, OscPacket, OscType};

extern crate hex;

const GOLDEN_MESSAGE_WO_ARGS: &str = "2f736f6d652f6164647200002c000000";
const GOLDEN_MESSAGE_WITH_ALL_TYPES: &str = "2f616e6f746865722f616464726573732f3100002c69686664737362746346544e496d725b695b64645d735d0000000000000004000000000000002a40490fda400921fb54442eea54686973206973206120737472696e672e00000054686973206973206120737472696e6720746f6f2e00000000000003010203000000007b000001c80000006304292a81ffc02a0d0000002a3ff3ae147ae147ae4009ae147ae147ae59617900";
const GOLDEN_EMPTY_BUNDLE: &str = "2362756e646c65000000000400000002";
const GOLDEN_BUNDLE: &str = "2362756e646c6500000004d2000010e10000000c2f766965772f31002c000000000000202f6d697865722f6368616e6e656c2f312f616d70000000002c6600003f666666000000442362756e646c65000000162e0000223d000000142f6f73632f312f66726571002c690000000001b8000000182f6f73632f312f7068617365000000002c660000becccccd";
const GOLDEN_MULTI_PACKET: &str = "000000102f736f6d652f6164647200002c0000000000008c2362756e646c6500000004d2000010e10000000c2f766965772f31002c000000000000202f6d697865722f6368616e6e656c2f312f616d70000000002c6600003f666666000000442362756e646c65000000162e0000223d000000142f6f73632f312f66726571002c690000000001b8000000182f6f73632f312f7068617365000000002c660000becccccd";
const GOLDEN_MULTI_PACKET_BUNDLE_FIRST: &str = "0000008c2362756e646c6500000004d2000010e10000000c2f766965772f31002c000000000000202f6d697865722f6368616e6e656c2f312f616d70000000002c6600003f666666000000442362756e646c65000000162e0000223d000000142f6f73632f312f66726571002c690000000001b8000000182f6f73632f312f7068617365000000002c660000becccccd000000102f736f6d652f6164647200002c000000";

fn prefix_with_size(size: u32, golden: &str) -> String {
    [hex::encode(size.to_be_bytes()), golden.into()].concat()
}

#[test]
fn test_message_wo_args() {
    let packet = OscPacket::Message(OscMessage {
        addr: "/some/addr".to_string(),
        args: vec![],
    });

    // Datagram encoding and decoding.
    let bytes = encoder::encode(&packet).expect("encode failed");
    assert_eq!(hex::decode(GOLDEN_MESSAGE_WO_ARGS).unwrap(), bytes);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet);

    // Stream encoding and decoding.
    let bytes = encoder::encode_tcp(&[packet.clone()]).expect("stream encode failed");
    assert_eq!(
        prefix_with_size(16, GOLDEN_MESSAGE_WO_ARGS),
        hex::encode(&bytes)
    );

    let (tail, decoded_packet) = decoder::decode_tcp(&bytes).expect("stream decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(Some(packet), decoded_packet);
}

#[test]
fn test_encode_message_with_all_types() {
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

    // Datagram encoding and decoding.
    let bytes = encoder::encode(&packet).expect("encode failed");
    assert_eq!(hex::decode(GOLDEN_MESSAGE_WITH_ALL_TYPES).unwrap(), bytes);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet);

    // Stream encoding and decoding.
    let bytes = encoder::encode_tcp(&[packet.clone()]).expect("stream encode failed");
    assert_eq!(
        prefix_with_size(168, GOLDEN_MESSAGE_WITH_ALL_TYPES),
        hex::encode(&bytes)
    );

    let (tail, decoded_packet) = decoder::decode_tcp(&bytes).expect("stream decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(Some(packet), decoded_packet);
}

#[test]
fn test_empty_bundle() {
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (4, 2).into(),
        content: vec![],
    });

    // Datagram encoding and decoding.
    let bytes = encoder::encode(&packet).expect("encode failed");
    assert_eq!(hex::decode(GOLDEN_EMPTY_BUNDLE).unwrap(), bytes);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet);

    // Stream encoding and decoding.
    let bytes = encoder::encode_tcp(&[packet.clone()]).expect("stream encode failed");
    assert_eq!(
        prefix_with_size(16, GOLDEN_EMPTY_BUNDLE),
        hex::encode(&bytes)
    );

    let (tail, decoded_packet) = decoder::decode_tcp(&bytes).expect("stream decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(Some(packet), decoded_packet);
}

#[test]
fn test_bundle() {
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

    // Datagram encoding and decoding.
    let bytes = encoder::encode(&packet).expect("encode failed");
    assert_eq!(hex::decode(GOLDEN_BUNDLE).unwrap(), bytes);

    let (tail, decoded_packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(packet, decoded_packet);

    // Stream encoding and decoding.
    let bytes = encoder::encode_tcp(&[packet.clone()]).expect("stream encode failed");
    assert_eq!(prefix_with_size(140, GOLDEN_BUNDLE), hex::encode(&bytes));

    let (tail, decoded_packet) = decoder::decode_tcp(&bytes).expect("stream decode failed");
    assert_eq!(0, tail.len());
    assert_eq!(Some(packet), decoded_packet);
}

#[test]
fn test_multi_packet_message() {
    let packets = vec![
        OscPacket::Message(OscMessage {
            addr: "/some/addr".to_string(),
            args: vec![],
        }),
        OscPacket::Bundle(OscBundle {
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
        }),
    ];

    // Stream encoding and decoding.
    let bytes = encoder::encode_tcp(&packets).expect("stream encode failed");
    assert_eq!(GOLDEN_MULTI_PACKET, hex::encode(&bytes));

    let (remainder, decoded_packets) =
        decoder::decode_tcp_vec(&bytes).expect("stream decode failed");
    assert!(remainder.is_empty());
    assert_eq!(packets, decoded_packets);
}

#[test]
fn test_multi_packet_message_bundle_first() {
    let packets = vec![
        OscPacket::Bundle(OscBundle {
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
        }),
        OscPacket::Message(OscMessage {
            addr: "/some/addr".to_string(),
            args: vec![],
        }),
    ];

    // Stream encoding and decoding.
    let bytes = encoder::encode_tcp(&packets).expect("stream encode failed");
    assert_eq!(GOLDEN_MULTI_PACKET_BUNDLE_FIRST, hex::encode(&bytes));

    let (remainder, decoded_packets) =
        decoder::decode_tcp_vec(&bytes).expect("stream decode failed");
    assert!(remainder.is_empty());
    // Note that OSC packets following an OSC bundle get included into the OSC bundles contents.
    assert_eq!(
        vec![OscPacket::Bundle(OscBundle {
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
                OscPacket::Message(OscMessage {
                    addr: "/some/addr".to_string(),
                    args: vec![],
                }),
            ],
        }),],
        decoded_packets
    );
}

#[cfg(feature = "std")]
#[test]
fn test_bundle_cursor() {
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
    let n = encoder::encode_into(
        &packet,
        &mut encoder::WriteOutput(std::io::Cursor::new(&mut bytes)),
    )
    .expect("encode failed");
    assert_eq!(140, n);
    assert_eq!(hex::decode(GOLDEN_BUNDLE).unwrap(), bytes);
}
