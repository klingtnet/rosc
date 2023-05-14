extern crate byteorder;
extern crate rosc;

use byteorder::{BigEndian, ByteOrder};

use rosc::{decoder, encoder, OscBundle, OscPacket, OscTime, OscType};

#[test]
fn test_decode_udp_no_args() {
    let raw_addr = "/some/valid/address/4";
    let addr = encoder::encode_string(raw_addr);
    let type_tags = encoder::encode_string(",");
    let bytes: Vec<u8> = addr.into_iter().chain(type_tags.into_iter()).collect();
    let (remainder, packet) = decoder::decode_udp(&bytes).expect("decode failed");

    assert_eq!(remainder.len(), 0);
    assert_eq!(
        OscPacket::Message(rosc::OscMessage {
            addr: raw_addr.into(),
            args: vec![],
        }),
        packet,
    );
}

#[test]
fn test_decode_tcp_no_args() {
    let raw_addr = "/some/valid/address/4";
    let addr = encoder::encode_string(raw_addr);
    let type_tags = encoder::encode_string(",");
    let mut bytes: Vec<u8> = addr.into_iter().chain(type_tags.into_iter()).collect();

    let packet_size_header = (bytes.len() as u32).to_be_bytes().to_vec();
    bytes = vec![packet_size_header, bytes].concat();

    let (remainder, packet) = decoder::decode_tcp_vec(&bytes).expect("decode failed");

    assert_eq!(remainder.len(), 0);
    assert_eq!(
        vec![OscPacket::Message(rosc::OscMessage {
            addr: raw_addr.into(),
            args: vec![],
        }),],
        packet,
    )
}

#[test]
fn test_decode_udp_empty_bundle() {
    let timetag = OscTime::from((4, 2));
    let content = vec![];
    let bytes = encoder::encode(&OscPacket::Bundle(OscBundle {
        timetag: timetag,
        content: content.clone(),
    }))
    .expect("encode failed");
    let (remainder, packet) = decoder::decode_udp(&bytes).expect("decode failed");

    assert_eq!(remainder.len(), 0);
    assert_eq!(OscPacket::Bundle(OscBundle { timetag, content }), packet)
}

#[test]
fn test_decode_udp_args() {
    let addr = encoder::encode_string("/another/valid/address/123");
    let f = 3.1415f32;
    let mut f_bytes: [u8; 4] = [0u8; 4];
    BigEndian::write_f32(&mut f_bytes, f);
    assert_eq!(BigEndian::read_f32(&f_bytes), f);

    let d = 3.14159265359f64;
    let mut d_bytes: [u8; 8] = [0u8; 8];
    BigEndian::write_f64(&mut d_bytes, d);
    assert_eq!(BigEndian::read_f64(&d_bytes), d);

    let i = 12345678i32;
    let i_bytes: [u8; 4] = i.to_be_bytes();

    let l = -1234567891011i64;
    let h_bytes: [u8; 8] = l.to_be_bytes();

    let blob_size: [u8; 4] = 6u32.to_be_bytes();
    let blob: Vec<u8> = vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8];

    let s = "I am an osc test string.";
    assert!(s.is_ascii());
    let s_bytes: Vec<u8> = encoder::encode_string(s);

    let c = '$';
    let c_bytes: [u8; 4] = (c as u32).to_be_bytes();

    let a = vec![OscType::Int(i), OscType::Float(f), OscType::Int(i)];

    let type_tags = encoder::encode_string(",fdsTFibhNIc[ifi]");

    let args: Vec<u8> = f_bytes
        .iter()
        .chain(d_bytes.iter())
        .chain(s_bytes.iter())
        .chain(i_bytes.iter())
        .chain(blob_size.iter())
        .chain(blob.iter())
        .chain(vec![0u8, 0u8].iter())
        .chain(h_bytes.iter())
        .chain(c_bytes.iter())
        // array content
        .chain(i_bytes.iter())
        .chain(f_bytes.iter())
        .chain(i_bytes.iter())
        .copied()
        .collect::<Vec<u8>>();

    let bytes: Vec<u8> = addr
        .into_iter()
        .chain(type_tags.into_iter())
        .chain(args)
        .collect::<Vec<u8>>();

    let (remainder, packet) = decoder::decode_udp(&bytes).expect("decode failed");
    assert_eq!(remainder.len(), 0);
    assert_eq!(
        OscPacket::Message(rosc::OscMessage {
            addr: "/another/valid/address/123".into(),
            args: vec![
                rosc::OscType::Float(f),
                rosc::OscType::Double(d),
                rosc::OscType::String(s.into()),
                rosc::OscType::Bool(true),
                rosc::OscType::Bool(false),
                rosc::OscType::Int(i),
                rosc::OscType::Blob(blob),
                rosc::OscType::Long(l),
                rosc::OscType::Nil,
                rosc::OscType::Inf,
                rosc::OscType::Char(c),
                rosc::OscType::Array(rosc::OscArray { content: a }),
            ]
        }),
        packet
    )
}
