use crate::errors;
#[cfg(feature = "std")]
use core::fmt::{self, Display};
use core::{iter::FromIterator, result};

#[cfg(feature = "std")]
use std::{
    convert::{TryFrom, TryInto},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// A time tag in OSC message consists of two 32-bit integers where the first one denotes the number of seconds since 1900-01-01 and the second the fractions of a second.
/// For details on its semantics see <http://opensoundcontrol.org/node/3/#timetags>
///
/// # Examples
///
/// ```
/// #[cfg(feature = "std")]
/// {
///     use rosc::OscTime;
///     use std::{convert::TryFrom, time::UNIX_EPOCH};
///
///     assert_eq!(
///         OscTime::try_from(UNIX_EPOCH).unwrap(),
///         OscTime::from((2_208_988_800, 0))
///     );
/// }
/// ```
///
/// # Conversions between `(u32, u32)`
///
/// Prior to version `0.5.0` of this crate, `OscTime` was defined as a type alias to `(u32, u32)`.
/// If you are upgrading from one of these older versions, you can use [`.into()`](Into::into) to
/// convert between `(u32, u32)` and `OscTime` in either direction.
///
/// # Conversions between [`std::time::SystemTime`]
///
/// The traits in `std::convert` are implemented for converting between
/// [`SystemTime`](std::time::SystemTime) and `OscTime` in both directions. An `OscTime` can be
/// converted into a `SystemTime` using [`From`](std::convert::From)/[`Into`](std::convert::Into).
/// A `SystemTime` can be converted into an `OscTime` using
/// [`TryFrom`](std::convert::TryFrom)/[`TryInto`](std::convert::TryInto). The fallible variants of
/// the conversion traits are used this case because not every `SystemTime` can be represented as
/// an `OscTime`.
///
/// **These conversions are lossy**, but are tested to have a deviation within
/// 5 nanoseconds when converted back and forth in either direction.
///
/// Although any time since the OSC epoch (`1900-01-01 00:00:00 UTC`) can be represented using the
/// OSC timestamp format, this crate only allows conversions between times greater than or equal to
/// the [`UNIX_EPOCH`](std::time::UNIX_EPOCH). This allows the math used in the conversions to work
/// on 32-bit systems which cannot represent times that far back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OscTime {
    pub seconds: u32,
    pub fractional: u32,
}

#[cfg(feature = "std")]
impl OscTime {
    const UNIX_OFFSET: u64 = 2_208_988_800; // From RFC 5905
    const TWO_POW_32: f64 = (u32::MAX as f64) + 1.0; // Number of bits in a `u32`
    const ONE_OVER_TWO_POW_32: f64 = 1.0 / OscTime::TWO_POW_32;
    const NANOS_PER_SECOND: f64 = 1.0e9;
    const SECONDS_PER_NANO: f64 = 1.0 / OscTime::NANOS_PER_SECOND;
}

#[cfg(feature = "std")]
impl TryFrom<SystemTime> for OscTime {
    type Error = OscTimeError;

    fn try_from(time: SystemTime) -> core::result::Result<OscTime, OscTimeError> {
        let duration_since_epoch = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OscTimeError(OscTimeErrorKind::BeforeEpoch))?
            + Duration::new(OscTime::UNIX_OFFSET, 0);
        let seconds = u32::try_from(duration_since_epoch.as_secs())
            .map_err(|_| OscTimeError(OscTimeErrorKind::Overflow))?;
        let nanos = duration_since_epoch.subsec_nanos() as f64;
        let fractional = (nanos * OscTime::SECONDS_PER_NANO * OscTime::TWO_POW_32).round() as u32;
        Ok(OscTime {
            seconds,
            fractional,
        })
    }
}

#[cfg(feature = "std")]
impl From<OscTime> for SystemTime {
    fn from(time: OscTime) -> SystemTime {
        let nanos =
            (time.fractional as f64) * OscTime::ONE_OVER_TWO_POW_32 * OscTime::NANOS_PER_SECOND;
        let duration_since_osc_epoch = Duration::new(time.seconds as u64, nanos.round() as u32);
        let duration_since_unix_epoch =
            duration_since_osc_epoch - Duration::new(OscTime::UNIX_OFFSET, 0);
        UNIX_EPOCH + duration_since_unix_epoch
    }
}

impl From<(u32, u32)> for OscTime {
    fn from(time: (u32, u32)) -> OscTime {
        let (seconds, fractional) = time;
        OscTime {
            seconds,
            fractional,
        }
    }
}

impl From<OscTime> for (u32, u32) {
    fn from(time: OscTime) -> (u32, u32) {
        (time.seconds, time.fractional)
    }
}

#[cfg(feature = "std")]
/// An error returned by conversions involving [`OscTime`].
#[derive(Debug)]
pub struct OscTimeError(OscTimeErrorKind);

#[cfg(feature = "std")]
#[derive(Debug)]
enum OscTimeErrorKind {
    BeforeEpoch,
    Overflow,
}

#[cfg(feature = "std")]
impl Display for OscTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            OscTimeErrorKind::BeforeEpoch => {
                write!(f, "time is before the unix epoch and cannot be stored")
            }
            OscTimeErrorKind::Overflow => {
                write!(f, "time overflows what OSC time can store")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OscTimeError {}

/// see OSC Type Tag String: [OSC Spec. 1.0](http://opensoundcontrol.org/spec-1_0)
/// padding: zero bytes (n*4)
#[derive(Clone, Debug, PartialEq)]
pub enum OscType {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
    // use struct for time tag to avoid destructuring
    Time(OscTime),
    Long(i64),
    Double(f64),
    Char(char),
    Color(OscColor),
    Midi(OscMidiMessage),
    Bool(bool),
    Array(OscArray),
    Nil,
    Inf,
}
macro_rules! value_impl {
    ($(($name:ident, $variant:ident, $ty:ty)),*) => {
        $(
        impl OscType {
            #[allow(dead_code)]
            pub fn $name(self) -> Option<$ty> {
                match self {
                    OscType::$variant(v) => Some(v),
                    _ => None
                }
            }
        }
        impl From<$ty> for OscType {
            fn from(v: $ty) -> Self {
                OscType::$variant(v)
            }
        }
        )*
    }
}
value_impl! {
    (int, Int, i32),
    (float, Float, f32),
    (string, String, String),
    (blob, Blob, Vec<u8>),
    (array, Array, OscArray),
    (long, Long, i64),
    (double, Double, f64),
    (char, Char, char),
    (color, Color, OscColor),
    (midi, Midi, OscMidiMessage),
    (bool, Bool, bool)
}
impl From<(u32, u32)> for OscType {
    fn from(time: (u32, u32)) -> Self {
        OscType::Time(time.into())
    }
}

#[cfg(feature = "std")]
impl TryFrom<SystemTime> for OscType {
    type Error = OscTimeError;

    fn try_from(time: SystemTime) -> std::result::Result<OscType, OscTimeError> {
        time.try_into().map(OscType::Time)
    }
}

impl OscType {
    pub fn time(self) -> Option<OscTime> {
        match self {
            OscType::Time(time) => Some(time),
            _ => None,
        }
    }
}
impl<'a> From<&'a str> for OscType {
    fn from(string: &'a str) -> Self {
        OscType::String(string.to_string())
    }
}
/// Represents the parts of a Midi message. Mainly used for
/// tunneling midi over a network using the OSC protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OscMidiMessage {
    pub port: u8,
    pub status: u8,
    pub data1: u8, // maybe use an enum for data?
    pub data2: u8,
}

/// An *osc packet* can contain an *osc message* or a bundle of nested messages
/// which is called *osc bundle*.
#[derive(Clone, Debug, PartialEq)]
pub enum OscPacket {
    Message(OscMessage),
    Bundle(OscBundle),
}

/// An OSC message consists of an address and
/// zero or more arguments. The address should
/// specify an element of your Instrument (or whatever
/// you want to control with OSC) and the arguments
/// are used to set properties of the element to the
/// respective values.
#[derive(Clone, Debug, PartialEq)]
pub struct OscMessage {
    pub addr: String,
    pub args: Vec<OscType>,
}

/// An OSC bundle contains zero or more OSC packets
/// and a time tag. The contained packets *should* be
/// applied at the given time tag.
#[derive(Clone, Debug, PartialEq)]
pub struct OscBundle {
    pub timetag: OscTime,
    pub content: Vec<OscPacket>,
}

/// An RGBA color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OscColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

/// An OscArray color.
#[derive(Clone, Debug, PartialEq)]
pub struct OscArray {
    pub content: Vec<OscType>,
}

impl<T: Into<OscType>> FromIterator<T> for OscArray {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> OscArray {
        OscArray {
            content: iter.into_iter().map(T::into).collect(),
        }
    }
}

pub type Result<T> = result::Result<T, errors::OscError>;

impl From<String> for OscMessage {
    fn from(s: String) -> OscMessage {
        OscMessage {
            addr: s,
            args: vec![],
        }
    }
}
impl<'a> From<&'a str> for OscMessage {
    fn from(s: &str) -> OscMessage {
        OscMessage {
            addr: s.to_string(),
            args: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use super::*;
    #[cfg(feature = "std")]
    use std::time::UNIX_EPOCH;

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
    fn osc_times_can_be_converted_to_and_from_system_times() {
        let mut times = vec![];
        // Sweep across a few numbers to check for tolerance
        for seconds in vec![
            // We don't start at zero because times before the UNIX_EPOCH cannot be converted to
            // OscTime.
            OscTime::UNIX_OFFSET as u32,
            OscTime::UNIX_OFFSET as u32 + 1,
            OscTime::UNIX_OFFSET as u32 + 2,
            OscTime::UNIX_OFFSET as u32 + 3,
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
    #[test]
    fn osc_time_cannot_represent_times_before_1970_01_01() {
        assert!(OscTime::try_from(UNIX_EPOCH - Duration::from_secs(1)).is_err())
    }

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
    fn assert_eq_osc_times(a: OscTime, b: OscTime) {
        // I did not want to implement subtraction with carrying in order to implement this in the
        // same way as the alternative for system times. Intsead we are compare each part of the
        // OSC times separately.
        let tolerance_fractional_seconds = ((TOLERANCE_NANOS as f64 * OscTime::TWO_POW_32)
            / OscTime::NANOS_PER_SECOND)
            .round() as i64;
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
}
