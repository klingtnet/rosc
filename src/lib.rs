extern crate byteorder;

/// OSC data types, see [OSC 1.0
/// specification](http://opensoundcontrol.org/spec-1_0) for details
pub mod osc_types;
/// Everything that receives OSC messages is called an *OSC server*
pub mod osc_server;
/// Utilities, g.e. IP address parsing
pub mod utils;
/// Contains custom errors
pub mod errors;

#[test]
fn it_works() {
}
