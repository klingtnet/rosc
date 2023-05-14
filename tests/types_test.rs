extern crate rosc;

use rosc::{OscArray, OscTime, OscType};

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
    // same way as the alternative for system times. Intsead we are compare each part of the
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
