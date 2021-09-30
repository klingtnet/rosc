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


#[test]
pub fn test_match_address() {
    // Test invalid message and method address
    match address::match_address(&String::from("asd"), &String::from("/foo")).err().expect("") {
        OscError::BadAddress(_) => assert!(true),
        _ => assert!(false)
    }
    // Test invalid message address
    match address::match_address(&String::from("/foo"), &String::from("asd")).err().expect("") {
        OscError::BadAddress(_) => assert!(true),
        _ => assert!(false)
    }

    // Trivial match
    assert!(address::match_address(&String::from("/foo"), &String::from("/foo")).expect(""));
    assert!(address::match_address(&String::from("/foo/bar"), &String::from("/foo/bar")).expect(""));

    // Match with wildcard
    assert!(address::match_address(&String::from("/foo/*"), &String::from("/foo/bar")).expect(""));
    assert!(address::match_address(&String::from("/foo/b*r"), &String::from("/foo/baaaaaaaaaaaar")).expect(""));
    assert!(address::match_address(&String::from("/foo/b*r"), &String::from("/foo/barxxxxxxxxxxr")).expect(""));
    assert!(address::match_address(&String::from("/foo/b*r/baz"), &String::from("/foo/baaaaaaaaaaaar/baz")).expect(""));

    // Single character wildcard
    assert!(address::match_address(&String::from("/foo/b?r"), &String::from("/foo/bar")).expect(""));
    assert!(address::match_address(&String::from("/foo/b???r"), &String::from("/foo/baaar")).expect(""));

    // Combine wildcards
    assert!(address::match_address(&String::from("/foo/b?*?r"), &String::from("/foo/baar")).expect(""));
    assert!(address::match_address(&String::from("/foo/b?*?r"), &String::from("/foo/baaaar")).expect(""));

    // String lists
    assert!(address::match_address(&String::from("/foo/{bar,baz}"), &String::from("/foo/bar")).expect(""));
    assert!(address::match_address(&String::from("/foo/{bar,baz}"), &String::from("/foo/baz")).expect(""));
    assert!(!address::match_address(&String::from("/foo/{bar,baz}"), &String::from("/foo/bot")).expect(""));

    // Character ranges
    assert!(address::match_address(&String::from("/foo/b[a-c]r"), &String::from("/foo/bar")).expect(""));
    assert!(address::match_address(&String::from("/foo/b[a-c]r"), &String::from("/foo/bbr")).expect(""));
    assert!(address::match_address(&String::from("/foo/b[a-c]r"), &String::from("/foo/bcr")).expect(""));
    assert!(address::match_address(&String::from("/foo/b[!a-c]r"), &String::from("/foo/bdr")).expect(""));

    // Test that address part matches full address, not just a subset, when using regex
    assert!(address::match_address(&String::from("/foo/b[ab]r"), &String::from("/foo/bar")).expect(""));
    assert!(!address::match_address(&String::from("/foo/b[ab]r"), &String::from("/foo/barasd")).expect(""));
    assert!(!address::match_address(&String::from("/foo/a?d"), &String::from("/foo/barasd")).expect(""));
}