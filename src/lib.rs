extern crate byteorder;

/// OSC data types, see [OSC 1.0
/// specification](http://opensoundcontrol.org/spec-1_0) for details
pub mod osc_types;
/// Provides a decoding method for OSC packets
pub mod osc_decoder;
/// Utilities, g.e. IP address parsing
pub mod utils;
/// Contains custom errors
pub mod errors;

#[test]
fn it_works() {
}
