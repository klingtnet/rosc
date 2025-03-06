extern crate rosc;

use rosc::{OscArray, OscType};

#[cfg(feature = "std")]
use rosc::{OscBundle, OscColor, OscMessage, OscMidiMessage, OscPacket, OscTime};

#[cfg(feature = "std")]
use std::{
    convert::TryFrom,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[test]
fn test_osc_array_from_iter() {
    use std::iter::FromIterator;
    let iter = (0..3).map(OscType::Int);
    let osc_arr = OscArray::from_iter(iter);
    assert_eq!(
        osc_arr,
        OscArray {
            content: vec![OscType::Int(0), OscType::Int(1), OscType::Int(2)]
        }
    );
}

#[cfg(feature = "std")]
#[cfg(target_os = "windows")]
// On Windows, the resolution of SystemTime is 100ns, as opposed to 1ns on UNIX
// (https://doc.rust-lang.org/std/time/struct.SystemTime.html#platform-specific-behavior).
//
// As a result, any conversion of OscTime to SystemTime results in the latter being quantized
// to the nearest 100ns (rounded down).
// This also means both types of round-trips are lossy.
const TOLERANCE_NANOS: u64 = 100;

#[cfg(feature = "std")]
#[cfg(not(target_os = "windows"))]
const TOLERANCE_NANOS: u64 = 5;

#[cfg(feature = "std")]
fn assert_eq_system_times(a: SystemTime, b: SystemTime) {
    let difference = if a < b {
        b.duration_since(a).unwrap()
    } else {
        a.duration_since(b).unwrap()
    };

    let tolerance = Duration::from_nanos(TOLERANCE_NANOS);

    if difference > tolerance {
        panic!(
            "the fractional seconds components of {:?} and {:?} vary more than the required tolerance of {:?}",
            a, b, tolerance,
        );
    }
}

#[cfg(feature = "std")]
#[test]
fn system_times_can_be_converted_to_and_from_osc() {
    let times = vec![UNIX_EPOCH, SystemTime::now()];
    for time in times {
        for i in 0..1000 {
            let time = time + Duration::from_nanos(1) * i;
            assert_eq_system_times(time, SystemTime::from(OscTime::try_from(time).unwrap()));
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn osc_time_cannot_represent_times_before_1970_01_01() {
    assert!(OscTime::try_from(UNIX_EPOCH - Duration::from_secs(1)).is_err())
}

#[cfg(feature = "std")]
#[test]
fn osc_times_can_be_converted_to_and_from_system_times() {
    const UNIX_OFFSET: u64 = 2_208_988_800;

    let mut times = vec![];
    // Sweep across a few numbers to check for tolerance
    for seconds in vec![
        // We don't start at zero because times before the UNIX_EPOCH cannot be converted to
        // OscTime.
        UNIX_OFFSET as u32,
        UNIX_OFFSET as u32 + 1,
        UNIX_OFFSET as u32 + 2,
        UNIX_OFFSET as u32 + 3,
        u32::MAX - 1,
        u32::MAX,
    ] {
        let fractional_max = 100;
        for fractional in 0..fractional_max {
            times.push((seconds, fractional));
            times.push((seconds, fractional_max - fractional));
        }
    }

    for osc_time in times.into_iter().map(OscTime::from) {
        assert_eq_osc_times(
            osc_time,
            OscTime::try_from(SystemTime::from(osc_time)).unwrap(),
        );
    }
}

#[cfg(feature = "std")]
fn assert_eq_osc_times(a: OscTime, b: OscTime) {
    const TWO_POW_32: f64 = (u32::MAX as f64) + 1.0;
    const NANOS_PER_SECOND: f64 = 1.0e9;

    // I did not want to implement subtraction with carrying in order to implement this in the
    // same way as the alternative for system times. Instead we are compare each part of the
    // OSC times separately.
    let tolerance_fractional_seconds =
        ((TOLERANCE_NANOS as f64 * TWO_POW_32) / NANOS_PER_SECOND).round() as i64;
    assert_eq!(
        a.seconds, b.seconds,
        "the seconds components of {:?} and {:?} are different",
        a, b
    );
    if (a.fractional as i64 - b.fractional as i64).abs() > tolerance_fractional_seconds {
        panic!(
            "the fractional seconds components of {:?} and {:?} vary more than the required tolerance of {} fractional seconds",
            a, b, tolerance_fractional_seconds,
        );
    }
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_int() {
    let arg = OscType::Int(123);
    assert_eq!(arg.to_string(), "(i) 123".to_string());
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_float() {
    let arg = OscType::Float(123.4);
    assert_osc_type_display_eq(&arg, "(f) 123.4");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_string() {
    let arg = OscType::String("abc".to_string());
    assert_osc_type_display_eq(&arg, "(s) abc");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_blob() {
    let arg = OscType::Blob(vec![0, 1, 2, 255]);
    assert_osc_type_display_eq(&arg, "(b) 0x000102FF");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_time() {
    let arg = OscType::Time(OscTime::try_from(UNIX_EPOCH).unwrap());
    assert_osc_type_display_eq(&arg, "(t) 1970-01-01T00:00:00.000000000Z");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_long() {
    let arg = OscType::Long(123);
    assert_osc_type_display_eq(&arg, "(h) 123");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_double() {
    let arg = OscType::Double(123.4);
    assert_osc_type_display_eq(&arg, "(d) 123.4");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_char() {
    let arg = OscType::Char('a');
    assert_osc_type_display_eq(&arg, "(c) a");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_color() {
    let arg = OscType::Color(OscColor {
        red: 255,
        green: 127,
        blue: 63,
        alpha: 255,
    });
    assert_osc_type_display_eq(&arg, "(r) {255,127,63,255}");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_midi() {
    let arg = OscType::Midi(OscMidiMessage {
        port: 3,
        status: 0xF0,
        data1: 0x12,
        data2: 0x34,
    });
    assert_osc_type_display_eq(&arg, "(m) {port:3, status:0xF0, data:0x1234}");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_bool() {
    let arg_true = OscType::Bool(true);
    assert_osc_type_display_eq(&arg_true, "(T)");

    let arg_false = OscType::Bool(false);
    assert_osc_type_display_eq(&arg_false, "(F)");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_array() {
    let arg = OscType::Array(OscArray {
        content: vec![
            OscType::Int(123),
            OscType::Float(123.4),
            OscType::String("abc".to_string()),
        ],
    });
    assert_osc_type_display_eq(&arg, "[(i) 123,(f) 123.4,(s) abc]");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_nil() {
    let arg = OscType::Nil;
    assert_osc_type_display_eq(&arg, "(N)");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_type_infinitum() {
    let arg = OscType::Inf;
    assert_osc_type_display_eq(&arg, "(I)");
}

#[cfg(feature = "std")]
#[test]
fn display_osc_packet_message() {
    let packet = OscPacket::Message(OscMessage {
        addr: "/oscillator/1/frequency".to_string(),
        args: vec![OscType::Float(123.4), OscType::Bool(true)],
    });
    assert_eq!(
        packet.to_string(),
        "/oscillator/1/frequency, (f) 123.4, (T)"
    )
}

#[cfg(feature = "std")]
#[test]
fn display_osc_packet_bundle() {
    let timetag = OscTime::try_from(UNIX_EPOCH).unwrap();
    let packet = OscPacket::Bundle(OscBundle {
        timetag,
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/oscillator/1/frequency".to_string(),
                args: vec![OscType::Float(123.4), OscType::Bool(true)],
            }),
            OscPacket::Message(OscMessage {
                addr: "/oscillator/2/frequency".to_string(),
                args: vec![OscType::Float(246.8), OscType::Bool(false)],
            }),
        ],
    });
    assert_eq!(
        packet.to_string(),
        "#bundle 1970-01-01T00:00:00.000000000Z { /oscillator/1/frequency, (f) 123.4, (T); \
         /oscillator/2/frequency, (f) 246.8, (F) }"
    )
}

#[cfg(feature = "std")]
#[test]
fn display_osc_packet_nested_bundle() {
    let timetag = OscTime::try_from(UNIX_EPOCH).unwrap();
    let packet = OscPacket::Bundle(OscBundle {
        timetag,
        content: vec![
            OscPacket::Message(OscMessage {
                addr: "/oscillator/1/frequency".to_string(),
                args: vec![OscType::Float(123.4), OscType::Bool(true)],
            }),
            OscPacket::Message(OscMessage {
                addr: "/oscillator/2/frequency".to_string(),
                args: vec![OscType::Float(246.8), OscType::Bool(false)],
            }),
            OscPacket::Bundle(OscBundle {
                timetag,
                content: vec![OscPacket::Message(OscMessage {
                    addr: "/oscillator/3/frequency".to_string(),
                    args: vec![OscType::Float(123.4), OscType::Bool(true)],
                })],
            }),
        ],
    });
    assert_eq!(
        packet.to_string(),
        "#bundle 1970-01-01T00:00:00.000000000Z { /oscillator/1/frequency, (f) 123.4, (T); \
        /oscillator/2/frequency, (f) 246.8, (F); #bundle 1970-01-01T00:00:00.000000000Z \
        { /oscillator/3/frequency, (f) 123.4, (T) } }"
    )
}

#[cfg(feature = "std")]
fn assert_osc_type_display_eq(arg: &OscType, expected: &str) {
    assert_eq!(arg.to_string(), expected.to_string());
}
