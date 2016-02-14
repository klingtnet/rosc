//! **rosc** is an implementation of the [OSC 1.0](http://opensoundcontrol.org/spec-1_0) protocol in pure Rust.
//!

extern crate byteorder;

/// OSC data types, see [OSC 1.0
/// specification](http://opensoundcontrol.org/spec-1_0) for details
pub mod types;
/// Provides a decoding method for OSC packets
pub mod decoder;
/// TODO
pub mod encoder;
/// Utilities, g.e. IP address parsing
pub mod utils;
/// Contains custom errors
pub mod errors;
