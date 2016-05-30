//! **rosc** is an implementation of the [OSC 1.0](http://opensoundcontrol.org/spec-1_0) protocol in pure Rust.
//!

extern crate byteorder;

/// OSC data types, see [OSC 1.0 specification](http://opensoundcontrol.org/spec-1_0) for details.
mod types;
/// Crate specific error types.
mod errors;

pub use types::*;
pub use errors::*;

/// Provides a decoding method for OSC packets.
pub mod decoder;
/// Encodes an `OscPacket` to a byte vector.
pub mod encoder;
