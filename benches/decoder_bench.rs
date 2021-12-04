#![feature(test)]
extern crate rosc;
extern crate test;

use self::test::Bencher;

#[bench]
fn bench_decode(b: &mut Bencher) {
    // The message was captured from the `ytterbium` lemur patch looks like this:
    // OSC Bundle: OscBundle { timetag: Time(0, 1), content: [Message(OscMessage { addr: "/OSCILLATORS/OSC2/ADSR/x", args: Some([Float(0.1234567), Float(0.1234567), Float(0.1234567), Float(0.1234567)]) })] }
    let raw_msg: [u8; 72] = [
        35, 98, 117, 110, 100, 108, 101, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 52, 47, 79, 83, 67,
        73, 76, 76, 65, 84, 79, 82, 83, 47, 79, 83, 67, 50, 47, 65, 68, 83, 82, 47, 122, 0, 0, 0,
        0, 44, 102, 102, 102, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    b.iter(|| rosc::decoder::decode_udp(&raw_msg).unwrap());
}
