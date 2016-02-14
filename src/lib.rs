//! **rosc** is an implementation of the [OSC 1.0](http://opensoundcontrol.org/spec-1_0) protocol in pure Rust.
//!

extern crate byteorder;

/// OSC data types, see [OSC 1.0
/// specification](http://opensoundcontrol.org/spec-1_0) for details.
pub mod types;
/// Provides a decoding method for OSC packets.
pub mod decoder;
/// Encodes an `OscPacket` to a byte vector.
pub mod encoder;
/// Misc. utilities.
pub mod utils;
/// Crate specific error types.
pub mod errors;
