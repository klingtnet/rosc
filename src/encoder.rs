use crate::alloc::{string::String, vec::Vec};
use crate::types::{OscBundle, OscMessage, OscPacket, OscTime, OscType};

/// Takes a reference to an OSC packet and returns
/// a byte vector on success. If the packet was invalid
/// an `OscError` is returned.
///
/// # Example
///
/// ```
/// use rosc::{OscPacket,OscMessage,OscType};
/// use rosc::encoder;
///
/// let packet = OscPacket::Message(OscMessage{
///         addr: "/greet/me".to_string(),
///         args: vec![OscType::String("hi!".to_string())]
///     }
/// );
/// assert!(encoder::encode(&packet).is_ok())
/// ```
pub fn encode(packet: &OscPacket) -> crate::types::Result<Vec<u8>> {
    let mut bytes = Vec::new();

    // NOTE: The Output implementation for Vec<u8> can't actually produce an error!
    encode_into(packet, &mut bytes).expect("Failed to write encoded packet into Vec");

    Ok(bytes)
}

/// Takes a reference to an OSC packet and writes the
/// encoded bytes to the given output. On success, the
/// number of bytes written will be returned. If an error
/// occurs during encoding, encoding will stop and the
/// error will be returned. Note that in that case, the
/// output may have been partially written!
///
/// NOTE: The OSC encoder will write output in small pieces
/// (as small as a single byte), so the output should be
/// buffered if write calls have a large overhead (e.g.
/// writing to a file).
///
/// # Example
///
/// ```
/// use rosc::{OscPacket,OscMessage,OscType};
/// use rosc::encoder;
///
/// let mut bytes = Vec::new();
/// let packet = OscPacket::Message(OscMessage{
///         addr: "/greet/me".to_string(),
///         args: vec![OscType::String("hi!".to_string())]
///     }
/// );
/// assert!(encoder::encode_into(&packet, &mut bytes).is_ok())
/// ```
pub fn encode_into<O: Output>(packet: &OscPacket, out: &mut O) -> Result<usize, O::Err> {
    match *packet {
        OscPacket::Message(ref msg) => encode_message(msg, out),
        OscPacket::Bundle(ref bundle) => encode_bundle(bundle, out),
    }
}

fn encode_message<O: Output>(msg: &OscMessage, out: &mut O) -> Result<usize, O::Err> {
    let mut written = encode_string_into(&msg.addr, out)?;

    written += out.write(b",")?;
    for arg in &msg.args {
        written += encode_arg_type(arg, out)?;
    }

    let padding = pad(written as u64 + 1) as usize - written;
    written += out.write(&[0u8; 4][..padding])?;

    for arg in &msg.args {
        written += encode_arg_data(arg, out)?;
    }

    Ok(written)
}

fn encode_bundle<O: Output>(bundle: &OscBundle, out: &mut O) -> Result<usize, O::Err> {
    let mut written = encode_string_into("#bundle", out)?;
    written += encode_time_tag_into(&bundle.timetag, out)?;

    for packet in &bundle.content {
        match *packet {
            OscPacket::Message(ref m) => {
                let length_mark = out.mark(4)?;

                let length = encode_message(m, out)?;
                out.place(length_mark, &(length as u32).to_be_bytes())?;

                written += 4 + length;
            }
            OscPacket::Bundle(ref b) => {
                let length_mark = out.mark(4)?;

                let length = encode_bundle(b, out)?;
                out.place(length_mark, &(length as u32).to_be_bytes())?;

                written += 4 + length;
            }
        }
    }

    Ok(written)
}

fn encode_arg_data<O: Output>(arg: &OscType, out: &mut O) -> Result<usize, O::Err> {
    match *arg {
        OscType::Int(x) => out.write(&x.to_be_bytes()),
        OscType::Long(x) => out.write(&x.to_be_bytes()),
        OscType::Float(x) => out.write(&x.to_be_bytes()),
        OscType::Double(x) => out.write(&x.to_be_bytes()),
        OscType::Char(x) => out.write(&(x as u32).to_be_bytes()),
        OscType::String(ref x) => encode_string_into(x, out),
        OscType::Blob(ref x) => {
            let padded_blob_length = pad(x.len() as u64) as usize;
            let padding = padded_blob_length - x.len();

            out.write(&(x.len() as u32).to_be_bytes())?;
            out.write(x)?;

            if padding > 0 {
                out.write(&[0u8; 3][..padding])?;
            }

            Ok(4 + padded_blob_length)
        }
        OscType::Time(ref time) => encode_time_tag_into(time, out),
        OscType::Midi(ref x) => out.write(&[x.port, x.status, x.data1, x.data2]),
        OscType::Color(ref x) => out.write(&[x.red, x.green, x.blue, x.alpha]),
        OscType::Bool(_) => Ok(0),
        OscType::Nil => Ok(0),
        OscType::Inf => Ok(0),
        OscType::Array(ref x) => {
            let mut written = 0;
            for v in &x.content {
                written += encode_arg_data(v, out)?;
            }
            Ok(written)
        }
    }
}

fn encode_arg_type<O: Output>(arg: &OscType, out: &mut O) -> Result<usize, O::Err> {
    match *arg {
        OscType::Int(_) => out.write(b"i"),
        OscType::Long(_) => out.write(b"h"),
        OscType::Float(_) => out.write(b"f"),
        OscType::Double(_) => out.write(b"d"),
        OscType::Char(_) => out.write(b"c"),
        OscType::String(_) => out.write(b"s"),
        OscType::Blob(_) => out.write(b"b"),
        OscType::Time(_) => out.write(b"t"),
        OscType::Midi(_) => out.write(b"m"),
        OscType::Color(_) => out.write(b"r"),
        OscType::Bool(x) => out.write(if x { b"T" } else { b"F" }),
        OscType::Nil => out.write(b"N"),
        OscType::Inf => out.write(b"I"),
        OscType::Array(ref x) => {
            let mut written = out.write(b"[")?;

            for v in &x.content {
                written += encode_arg_type(v, out)?;
            }

            written += out.write(b"]")?;
            Ok(written)
        }
    }
}

/// Null terminates the byte representation of string `s` and
/// adds null bytes until the length of the result is a
/// multiple of 4.
pub fn encode_string<S: Into<String>>(s: S) -> Vec<u8> {
    let mut bytes: Vec<u8> = s.into().into_bytes();

    let new_len = pad(bytes.len() as u64 + 1) as usize;
    bytes.resize(new_len, 0u8);

    bytes
}

/// Writes the given string `s` to the given Output, adding
/// 1-4 null bytes such that the length of the result is a
/// multiple of 4.
pub fn encode_string_into<S: AsRef<str>, O: Output>(s: S, out: &mut O) -> Result<usize, O::Err> {
    let s = s.as_ref();

    let padded_len = pad(s.len() as u64 + 1) as usize;
    let padding = padded_len - s.len();
    out.write(s.as_bytes())?;
    out.write(&[0u8; 4][..padding])?;
    Ok(s.len() + padding)
}

/// Returns the position padded to 4 bytes.
///
/// # Example
///
/// ```
/// use rosc::encoder;
///
/// let pos: u64 = 10;
/// assert_eq!(12u64, encoder::pad(pos))
/// ```
pub fn pad(pos: u64) -> u64 {
    match pos % 4 {
        0 => pos,
        d => pos + (4 - d),
    }
}

fn encode_time_tag_into<O: Output>(time: &OscTime, out: &mut O) -> Result<usize, O::Err> {
    out.write(&time.seconds.to_be_bytes())?;
    out.write(&time.fractional.to_be_bytes())?;
    Ok(8)
}

#[test]
fn test_pad() {
    assert_eq!(4, pad(4));
    assert_eq!(8, pad(5));
    assert_eq!(8, pad(6));
    assert_eq!(8, pad(7));
}

/// A trait for values that can receive encoded OSC output
/// via `encode_into`. This allows more flexibility in how
/// the output is handled, including reusing part of an
/// existing buffer or writing directly to an external sink
/// (e.g. a file).
///
/// Implementations are currently provided for this trait for:
/// - `Vec<u8>`: Data will be appended to the end of the Vec.
/// - `WriteOutput<W>` (with feature `std`): A wrapper that
///   allows data to be written to any type that implements
///   `std::io::Seek + std::io::Write`.
pub trait Output {
    /// The error type which is returned from Output functions.
    type Err;

    /// The type which should be used to indicate the location of a mark.
    type Mark;

    /// Writes a block of data to the output.
    ///
    /// Note that, unlike `std::io::Writo::write`, this
    /// function is expected to write all of the given data prior to returning.
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err>;

    /// Marks the location of a fixed-length value and returns a `Self::Mark` which may be used to
    /// fill in its data later with `place`.
    fn mark(&mut self, size: usize) -> Result<Self::Mark, Self::Err>;

    /// Consumes a previously-generated Mark and fills it in with data.
    ///
    /// This may result in a panic or in invalid data being written if `mark` came from a different
    /// `Output`, or if the length of `data` does not match the size passed to `mark`.
    fn place(&mut self, mark: Self::Mark, data: &[u8]) -> Result<(), Self::Err>;
}

impl Output for Vec<u8> {
    type Err = core::convert::Infallible;
    type Mark = (usize, usize);

    #[inline]
    fn mark(&mut self, size: usize) -> Result<Self::Mark, Self::Err> {
        let start = self.len();
        let end = start + size;

        self.resize(end, 0);
        Ok((start, end))
    }

    #[inline]
    fn place(&mut self, (start, end): Self::Mark, data: &[u8]) -> Result<(), Self::Err> {
        self[start..end].copy_from_slice(data);
        Ok(())
    }

    #[inline]
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err> {
        self.extend(data);
        Ok(data.len())
    }
}

/// A newtype which can be used to wrap any type which
/// implements `std::io::Seek` and `std::io::Write` to allow
/// it to be used as an `Output`.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WriteOutput<W>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Seek + std::io::Write> Output for WriteOutput<W> {
    type Err = std::io::Error;
    type Mark = u64;

    fn mark(&mut self, size: usize) -> Result<Self::Mark, Self::Err> {
        let pos = self.0.stream_position()?;

        let mut left = size;
        while left > 0 {
            let num = left.min(8);
            self.0.write_all(&[0; 8][..num])?;
            left -= num;
        }

        Ok(pos)
    }

    fn place(&mut self, pos: Self::Mark, data: &[u8]) -> Result<(), Self::Err> {
        let old_pos = self.0.stream_position()?;

        self.0.seek(std::io::SeekFrom::Start(pos))?;
        self.0.write_all(data)?;
        self.0.seek(std::io::SeekFrom::Start(old_pos))?;

        Ok(())
    }

    #[inline]
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Err> {
        std::io::Write::write_all(&mut self.0, data).map(|_| data.len())
    }
}
