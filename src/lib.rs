//! **rosc** is an implementation of the [OSC 1.0](http://opensoundcontrol.org/spec-1_0) protocol in pure Rust.
//!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as core;
#[cfg(feature = "std")]
#[macro_use]
extern crate std as alloc;

extern crate byteorder;
extern crate nom;

/// Crate specific error types.
mod errors;
/// OSC data types, see [OSC 1.0 specification](https://opensoundcontrol.stanford.edu/spec-1_0.html) for details.
mod types;

pub use crate::errors::*;
pub use crate::types::*;

/// Address checking and matching methods
#[cfg(feature = "std")]
pub mod address;
/// Provides a decoding method for OSC packets.
pub mod decoder;
/// Encodes an `OscPacket` to a byte vector.
pub mod encoder;
