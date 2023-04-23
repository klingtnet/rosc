#![feature(test)]
extern crate rosc;
extern crate test;

use self::test::Bencher;
use rosc::*;

#[bench]
fn bench_encode_args_array(b: &mut Bencher) {
    // Encoded message contains 100 argumnts, each of which is an Array containing 0-20 Int values.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Array".into(),
        args: (0..100)
            .map(|i| {
                OscArray {
                    content: (0..i % 20).map(OscType::from).collect(),
                }
                .into()
            })
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_blob(b: &mut Bencher) {
    // Encoded message contains 1000 argumnts, each of which is a Blob containing 0-20 bytes.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Blobs".into(),
        args: (0..1000)
            .map(|x| OscType::Blob((0..(x % 20) as u8).collect()))
            .collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_bool(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is a Bool. Half are false and half are
    // true.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Bools".into(),
        args: (0..1000).map(|x| OscType::Bool((x % 2) == 1)).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_double(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is a Double.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Doubles".into(),
        args: (0..1000).map(|x| OscType::Double(x as f64)).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_float(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is a Float.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Floats".into(),
        args: (0..1000).map(|x| OscType::Float(x as f32)).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_int(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is an Int.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Ints".into(),
        args: (0..1000).map(OscType::Int).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_long(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is a Long.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Longs".into(),
        args: (0..1000).map(OscType::Long).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_nil(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is Nil.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Nils".into(),
        args: (0..1000).map(|_| OscType::Nil).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_args_string(b: &mut Bencher) {
    // Encoded message contains 1000 arguments, each of which is a String containing the string
    // representation of its argument index.
    let packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Strings".into(),
        args: (0..1000).map(|x| OscType::String(x.to_string())).collect(),
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_bundles(b: &mut Bencher) {
    // Encoded bundle contains 1000 sub-bundles, each of which are empty.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Bundle(OscBundle {
                timetag: (0, 0).into(),
                content: vec![],
            });
            1000
        ],
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_bundles_into_new_vec(b: &mut Bencher) {
    // Encoded bundle contains 1000 sub-bundles, each of which are empty.
    // The packet is encoded into a new Vec each time, resulting in a fresh allocation.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Bundle(OscBundle {
                timetag: (0, 0).into(),
                content: vec![],
            });
            1000
        ],
    });

    b.iter(|| rosc::encoder::encode_into(&packet, &mut Vec::new()).unwrap());
}

#[bench]
fn bench_encode_bundles_into_reused_vec(b: &mut Bencher) {
    // Encoded bundle contains 1000 sub-bundles, each of which are empty.
    // The packet is encoded into the same Vec each time, resulting in no allocation after the first.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Bundle(OscBundle {
                timetag: (0, 0).into(),
                content: vec![],
            });
            1000
        ],
    });

    let mut buffer = Vec::new();
    b.iter(|| {
        buffer.clear();
        rosc::encoder::encode_into(&packet, &mut buffer).unwrap()
    });
}

#[bench]
fn bench_encode_huge_bundle(b: &mut Bencher) {
    // Encoded bundle contains 1000 messages, each of which contains an argument of every type
    // (including a 1 KB blob).
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/OSC/Message".into(),
                args: vec![
                    4i32.into(),
                    42i64.into(),
                    3.1415926f32.into(),
                    3.14159265359f64.into(),
                    "String".into(),
                    (0..1024).map(|x| x as u8).collect::<Vec<u8>>().into(),
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
                            "Another String".into(),
                        ],
                    }
                    .into(),
                ],
            });
            1000
        ],
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_huge_bundle_into_new_vec(b: &mut Bencher) {
    // Encoded bundle contains 1000 messages, each of which contains an argument of every type
    // (including a 1 KB blob).
    // The packet is encoded into a new Vec each time, resulting in a fresh allocation.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/OSC/Message".into(),
                args: vec![
                    4i32.into(),
                    42i64.into(),
                    3.1415926f32.into(),
                    3.14159265359f64.into(),
                    "String".into(),
                    (0..1024).map(|x| x as u8).collect::<Vec<u8>>().into(),
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
                            "Another String".into(),
                        ],
                    }
                    .into(),
                ],
            });
            1000
        ],
    });

    b.iter(|| rosc::encoder::encode_into(&packet, &mut Vec::new()).unwrap());
}

#[bench]
fn bench_encode_huge_bundle_into_reused_vec(b: &mut Bencher) {
    // Encoded bundle contains 1000 messages, each of which contains an argument of every type
    // (including a 1 KB blob).
    // The packet is encoded into the same Vec each time, resulting in no allocation after the first.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/OSC/Message".into(),
                args: vec![
                    4i32.into(),
                    42i64.into(),
                    3.1415926f32.into(),
                    3.14159265359f64.into(),
                    "String".into(),
                    (0..1024).map(|x| x as u8).collect::<Vec<u8>>().into(),
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
                            "Another String".into(),
                        ],
                    }
                    .into(),
                ],
            });
            1000
        ],
    });

    let mut buffer = Vec::new();
    b.iter(|| {
        buffer.clear();
        rosc::encoder::encode_into(&packet, &mut buffer).unwrap()
    });
}

#[bench]
fn bench_encode_messages(b: &mut Bencher) {
    // Encoded bundle contains 1000 messages, each of which has no arguments.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/OSC/Message".into(),
                args: vec![],
            });
            1000
        ],
    });

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}

#[bench]
fn bench_encode_messages_into_new_vec(b: &mut Bencher) {
    // Encoded bundle contains 1000 messages, each of which has no arguments.
    // The packet is encoded into a new Vec each time, resulting in a fresh allocation.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/OSC/Message".into(),
                args: vec![],
            });
            1000
        ],
    });

    b.iter(|| rosc::encoder::encode_into(&packet, &mut Vec::new()).unwrap());
}

#[bench]
fn bench_encode_messages_into_reused_vec(b: &mut Bencher) {
    // Encoded bundle contains 1000 messages, each of which has no arguments.
    // The packet is encoded into the same Vec each time, resulting in no allocation after the first.
    let packet = OscPacket::Bundle(OscBundle {
        timetag: (0, 0).into(),
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/OSC/Message".into(),
                args: vec![],
            });
            1000
        ],
    });

    let mut buffer = Vec::new();
    b.iter(|| {
        buffer.clear();
        rosc::encoder::encode_into(&packet, &mut buffer).unwrap()
    });
}

#[bench]
fn bench_encode_nested_bundles(b: &mut Bencher) {
    // Encoded bundle contains 1000 sub-bundles, each of which are empty.
    let mut packet = OscPacket::Message(OscMessage {
        addr: "/OSC/Nssted".into(),
        args: vec![],
    });

    for _ in 0..20 {
        packet = OscPacket::Bundle(OscBundle {
            timetag: (0, 0).into(),
            content: vec![packet],
        });
    }

    b.iter(|| rosc::encoder::encode(&packet).unwrap());
}
