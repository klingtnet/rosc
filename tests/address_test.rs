extern crate rosc;

use rosc::{OscError, address::Matcher};

#[test]
fn test_matcher() {
    let matcher = Matcher::new("/oscillator/[0-9]/*/pre[!1234?*]post/{frequency,phase}/x?")
        .expect("Matcher::new");
    assert_eq!(
        matcher
            .match_address("/oscillator/1/something/preXpost/phase/xy")
            .expect("should match"),
        true
    );
    assert_eq!(
        matcher
            .match_address("/oscillator/1/something/pre1post/phase/xy")
            .expect("should not match"),
        false
    );
    matcher.match_address("invalid_address").expect_err("should fail because address does not start with a slash");
}

#[test]
fn test_bad_address_pattern() {
    let expected_err = "bad OSC address pattern: bad address pattern";
    assert_eq!(Matcher::new("").unwrap_err().to_string(), expected_err);
    assert_eq!(Matcher::new("/").unwrap_err().to_string(), expected_err);
    assert_eq!(Matcher::new("//empty/parts/").unwrap_err().to_string(), expected_err);
    assert_eq!(Matcher::new("////").unwrap_err().to_string(), expected_err);
    assert_eq!(Matcher::new("/{unclosed,alternative").unwrap_err().to_string(), expected_err);
    assert_eq!(Matcher::new("/unclosed/[range-").unwrap_err().to_string(), expected_err);
}

#[test]
fn test_bad_address() {
    let matcher = Matcher::new("/does-not-matter").expect("Matcher::new");
    let expected_err = "bad OSC address: bad address";
    assert_eq!(matcher.match_address("").unwrap_err().to_string(), expected_err);
    assert_eq!(matcher.match_address("/").unwrap_err().to_string(), expected_err);
    assert_eq!(matcher.match_address("/contains/wildcards?").unwrap_err().to_string(), expected_err);
    assert_eq!(matcher.match_address("/contains/wildcards*").unwrap_err().to_string(), expected_err);
    assert_eq!(matcher.match_address("/contains/ranges[a-z]").unwrap_err().to_string(), expected_err);
    assert_eq!(matcher.match_address("/contains/ranges[!a-z]").unwrap_err().to_string(), expected_err);
    assert_eq!(matcher.match_address("/{contains,alternative}").unwrap_err().to_string(), expected_err);
}
