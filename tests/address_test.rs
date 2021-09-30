extern crate rosc;

use rosc::{address, OscError};

#[test]
fn test_validate_method_address() {
    // Valid addresses
    address::validate_method_address(&String::from("/")).expect("");
    address::validate_method_address(&String::from("/a")).expect("");

    // Invalid addresses
    match address::validate_method_address(&String::from("/foo\0")).err().expect("") {
        OscError::BadAddress("Address may only contain printable ASCII characters") => assert!(true),
        _ => assert!(false)
    }
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

#[test]
fn test_validate_message_address() {
    // Valid addresses
    address::validate_message_address(&String::from("/a")).expect("");
    address::validate_message_address(&String::from("/a[!a-z]")).expect("");
    address::validate_message_address(&String::from("/a{foo,bar}")).expect("");
    address::validate_message_address(&String::from("/a?/foo*/bar")).expect("");

    // Invalid addresses
    match address::validate_message_address(&String::from("/foo\0")).err().expect("") {
        OscError::BadAddress("Address may only contain printable ASCII characters") => assert!(true),
        _ => assert!(false)
    }
    match address::validate_message_address(&String::from("öäü")).err().expect("") {
        OscError::BadAddress("Address must only contain ASCII characters") => assert!(true),
        _ => assert!(false)
    }
    match address::validate_message_address(&String::from("")).err().expect("") {
        OscError::BadAddress("Address must be at least 1 character long") => assert!(true),
        _ => assert!(false)
    }
    match address::validate_message_address(&String::from("foo")).err().expect("") {
        OscError::BadAddress("Address must start with '/'") => assert!(true),
        _ => assert!(false)
    }

    // Square brackets open and never close
    match address::validate_message_address(&String::from("/[a/b")).err().expect("") {
        OscError::BadAddress("Character range (square brackets) was started but not closed before the next address part started") => assert!(true),
        _ => assert!(false)
    }
    // Square brackets contains /
    match address::validate_message_address(&String::from("/[a/]b")).err().expect("") {
        OscError::BadAddress("Character range (square brackets) was started but not closed before the next address part started") => assert!(true),
        _ => assert!(false)
    }
    // Square brackets open but don't close before the address ends
    match address::validate_message_address(&String::from("/[a")).err().expect("") {
        OscError::BadAddress("Character range (square brackets) was started but not closed before the end of the address") => assert!(true),
        _ => assert!(false)
    }

    // Curly brackets open and never close
    match address::validate_message_address(&String::from("/{a/b")).err().expect("") {
        OscError::BadAddress("String list (curly brackets) was started but not closed before the next address part started") => assert!(true),
        _ => assert!(false)
    }
    // Curly brackets contains /
    match address::validate_message_address(&String::from("/{a/}b")).err().expect("") {
        OscError::BadAddress("String list (curly brackets) was started but not closed before the next address part started") => assert!(true),
        _ => assert!(false)
    }
    // Curly brackets open but don't close before the address ends
    match address::validate_message_address(&String::from("/{a")).err().expect("") {
        OscError::BadAddress("String list (curly brackets) was started but not closed before the end of the address") => assert!(true),
        _ => assert!(false)
    }

    // Curly brackets within square brackets
    match address::validate_message_address(&String::from("/{a[foo]}/")).err().expect("") {
        OscError::BadAddress("Can not start a character range (square brackets) within string list (curly brackets)") => assert!(true),
        _ => assert!(false)
    }

    // Square brackets within curly brackets
    match address::validate_message_address(&String::from("/[a{foo}]/")).err().expect("") {
        OscError::BadAddress("Can not start a string list (curly brackets) within character range (square brackets)") => assert!(true),
        _ => assert!(false)
    }
}