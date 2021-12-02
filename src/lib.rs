//! **rosc** is an implementation of the [OSC 1.0](http://opensoundcontrol.org/spec-1_0) protocol in pure Rust.
//!

extern crate byteorder;
extern crate regex;
extern crate nom;

/// Crate specific error types.
mod errors;
/// OSC data types, see [OSC 1.0 specification](https://opensoundcontrol.stanford.edu/spec-1_0.html) for details.
mod types;

pub use crate::errors::*;
pub use crate::types::*;

/// Provides a decoding method for OSC packets.
pub mod decoder;
/// Encodes an `OscPacket` to a byte vector.
pub mod encoder;
/// Address checking and matching methods
pub mod address;
