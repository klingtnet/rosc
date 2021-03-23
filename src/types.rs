use crate::errors;
use std::{
    convert::{TryFrom, TryInto},
    error,
    fmt::{self, Display},
    iter::FromIterator,
    result,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// A time tag in OSC message consists of two 32-bit integers where the first one denotes the number of seconds since 1900-01-01 and the second the fractions of a second.
/// For details on its semantics see http://opensoundcontrol.org/node/3/#timetags
///
/// # Examples
///
/// ```
/// use rosc::OscTime;
/// use std::{convert::TryFrom, time::SystemTime};
///
/// assert_eq!(
///     OscTime::try_from(SystemTime::UNIX_EPOCH).unwrap(),
///     OscTime::from((2_208_988_800, 0))
/// );
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OscTime {
    pub seconds: u32,
    pub fractional: u32,
}

impl OscTime {
    const UNIX_OFFSET: u64 = 2_208_988_800; // From RFC 5905
    const TWO_POW_32: f64 = (u32::MAX as f64) + 1.0; // Number of bits in a `u32`
    const ONE_OVER_TWO_POW_32: f64 = 1.0 / OscTime::TWO_POW_32;
    const NANOS_PER_SECOND: f64 = 1.0e9;
    const SECONDS_PER_NANO: f64 = 1.0 / OscTime::NANOS_PER_SECOND;

    fn epoch() -> SystemTime {
        UNIX_EPOCH - Duration::from_secs(OscTime::UNIX_OFFSET)
    }
}

impl TryFrom<SystemTime> for OscTime {
    type Error = OscTimeError;

    fn try_from(time: SystemTime) -> std::result::Result<OscTime, OscTimeError> {
        let duration_since_epoch = time
            .duration_since(OscTime::epoch())
            .map_err(|_| OscTimeError(OscTimeErrorKind::BeforeEpoch))?;
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

impl From<OscTime> for SystemTime {
    fn from(time: OscTime) -> SystemTime {
        let nanos =
            (time.fractional as f64) * OscTime::ONE_OVER_TWO_POW_32 * OscTime::NANOS_PER_SECOND;
        let duration_since_osc_epoch = Duration::new(time.seconds as u64, nanos as u32);
        OscTime::epoch() + duration_since_osc_epoch
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

/// An error returned by conversions involving [`OscTime`].
#[derive(Debug)]
pub struct OscTimeError(OscTimeErrorKind);

#[derive(Debug)]
enum OscTimeErrorKind {
    BeforeEpoch,
    Overflow,
}

impl Display for OscTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            OscTimeErrorKind::BeforeEpoch => {
                write!(f, "time is before the OSC epoch and cannot be stored")
            }
            OscTimeErrorKind::Overflow => {
                write!(f, "time overflows what OSC time can store")
            }
        }
    }
}

impl error::Error for OscTimeError {}

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
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
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
    use super::*;

    const TOLERANCE_NANOS: u64 = 5;

    #[test]
    fn system_times_can_be_converted_to_and_from_osc() {
        let times = vec![UNIX_EPOCH, OscTime::epoch(), UNIX_EPOCH];
        for time in times {
            for i in 0..1000 {
                let time = time + Duration::from_nanos(1) * i;
                assert_eq_system_times(time, SystemTime::from(OscTime::try_from(time).unwrap()));
            }
        }
    }

    #[test]
    fn osc_times_can_be_converted_to_and_from_system_times() {
        let mut times = vec![];

        // Sweep across a few numbers to check for tolerance
        for seconds in vec![0, 1, 2, 3, u32::MAX - 1, u32::MAX] {
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

    #[test]
    fn osc_time_cannot_represent_times_before_1900_01_01() {
        assert!(OscTime::try_from(OscTime::epoch() - Duration::from_secs(1)).is_err())
    }

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
