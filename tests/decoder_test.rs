extern crate byteorder;
extern crate rosc;

use byteorder::{BigEndian, ByteOrder};

use rosc::{decoder, encoder, OscBundle, OscPacket, OscTime, OscType};

#[test]
fn test_decode_udp_no_args() {
    // message to build: /some/valid/address/4 ,
    let raw_addr = "/some/valid/address/4";
    let addr = encoder::encode_string(raw_addr);
    let type_tags = encoder::encode_string(",");
    let merged: Vec<u8> = addr.into_iter().chain(type_tags.into_iter()).collect();
    let (remainder, osc_packet) = decoder::decode_udp(&merged).unwrap();

    assert_eq!(remainder.len(), 0);
    match osc_packet {
        rosc::OscPacket::Message(msg) => {
            assert_eq!(raw_addr, msg.addr);
            assert!(msg.args.is_empty());
        }
        _ => panic!("Expected an OscMessage!"),
    }
}

#[test]
fn test_decode_tcp_vec() {
    use rosc::OscPacket::Message;

    // message to build: /some/valid/address/4 ,
    let raw_addr = "/some/valid/address/4";
    let addr = encoder::encode_string(raw_addr);
    let type_tags = encoder::encode_string(",");
    let merged: Vec<u8> = addr.into_iter().chain(type_tags.into_iter()).collect();

    let tcp_msg = std::iter::repeat_with(|| merged.clone())
        .take(2)
        .flat_map(|bytes| {
            // Prefix the tcp packet with a length byte
            let packet_size_header = (bytes.len() as u32).to_be_bytes().to_vec();
            vec![packet_size_header, bytes].concat()
        })
        .collect::<Vec<u8>>();

    let (remainder, osc_packet) = decoder::decode_tcp_vec(&tcp_msg).unwrap();

    assert_eq!(remainder.len(), 0);
    match &osc_packet[..] {
        [Message(msg1), Message(msg2)] => {
            assert_eq!(raw_addr, msg1.addr);
            assert!(msg1.args.is_empty());
            assert_eq!(raw_addr, msg2.addr);
            assert!(msg2.args.is_empty());
        }
        _ => panic!("Expected an OscMessage!"),
    }
}

#[test]
fn test_decode_udp_empty_bundle() {
    let timetag = OscTime::from((4, 2));
    let content = vec![];
    let packet = encoder::encode(&OscPacket::Bundle(OscBundle { timetag, content })).unwrap();
    let osc_packet = decoder::decode_udp(&packet);
    match osc_packet.unwrap().1 {
        rosc::OscPacket::Bundle(bundle) => {
            assert_eq!(timetag, bundle.timetag);
            assert!(bundle.content.is_empty());
        }
        _ => panic!("Expected an OscBundle!"),
    }
}

#[test]
fn test_decode_udp_args() {
    // /another/valid/address/123 ,fdih 3.1415 3.14159265359 12345678i32
    // -1234567891011
    let addr = encoder::encode_string("/another/valid/address/123");
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
    let i_bytes: [u8; 4] = i.to_be_bytes();

    let l = -1234567891011i64;
    let h_bytes: [u8; 8] = l.to_be_bytes();

    let blob_size: [u8; 4] = 6u32.to_be_bytes();
    let blob: Vec<u8> = vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8];

    let s = "I am an osc test string.";
    assert!(s.is_ascii());
    // Osc strings are null terminated like in C!
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

    let merged: Vec<u8> = addr
        .into_iter()
        .chain(type_tags.into_iter())
        .chain(args)
        .collect::<Vec<u8>>();

    match decoder::decode_udp(&merged).unwrap().1 {
        rosc::OscPacket::Message(msg) => {
            for arg in msg.args {
                match arg {
                    rosc::OscType::Int(x) => assert_eq!(i, x),
                    rosc::OscType::Long(x) => assert_eq!(l, x),
                    rosc::OscType::Float(x) => assert_eq!(f, x),
                    rosc::OscType::Double(x) => assert_eq!(d, x),
                    rosc::OscType::String(x) => assert_eq!(s, x),
                    rosc::OscType::Blob(x) => assert_eq!(blob, x),
                    // cant assign bool args to type_tag
                    // , so there is no real test wether the value is
                    // correct or not
                    rosc::OscType::Bool(_) => (),
                    rosc::OscType::Inf => (),
                    rosc::OscType::Nil => (),
                    // test time-tags, midi-messages and chars
                    rosc::OscType::Char(x) => assert_eq!(c, x),
                    rosc::OscType::Array(x) => assert_eq!(a, x.content),
                    _ => panic!(),
                }
            }
        }
        _ => panic!("Expected an OSC message!"),
    }
}
