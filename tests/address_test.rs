extern crate rosc;

use rosc::{address, OscError};

#[test]
fn test_validate_method_address() {
    // Valid addresses
    address::validate_method_address(&String::from("/"));
    address::validate_method_address(&String::from("/a"));

    // Invalid addresses
    match address::validate_method_address(&String::from("öäü")).err().expect("") {
        OscError::BadAddress("Address must only contain ASCII characters") => assert!(true),
        _ => assert!(false)
    }
    match address::validate_method_address(&String::from("")).err().expect("") {
        OscError::BadAddress("Address must be at least 1 character long") => assert!(true),
        _ => assert!(false)
    }
    match address::validate_method_address(&String::from("foo")).err().expect("") {
        OscError::BadAddress("Address must start with '/'") => assert!(true),
        _ => assert!(false)
    }
    match address::validate_method_address(&String::from("/ illegal#*,?[]{}")).err().expect("") {
        OscError::BadAddress("Address may not contain any of the following characters: ' #*,?[]{}'") => assert!(true),
        _ => assert!(false)
    }
}