extern crate rosc;

#[cfg(feature = "std")]
use rosc::address::Matcher;

#[cfg(feature = "std")]
#[test]
fn test_matcher() {
    let mut matcher;

    // Regular address using only alphanumeric parts
    matcher = Matcher::new("/oscillator/1/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/1/frequency").expect("Should match");
    matcher.match_address("/oscillator/1/phase").expect_err("Should not match");
    matcher.match_address("/oscillator/1/frequencyfoo").expect_err("Should not match");
    matcher.match_address("/prefix/oscillator/1/frequency").expect_err("Should not match");

    // Choice
    matcher = Matcher::new("/foo{bar,baz}").expect("Should be valid");
    matcher.match_address("/foobar").expect("Should match");
    matcher.match_address("/foobaz").expect("Should match");

    matcher = Matcher::new("/foo{bar,baz,tron}").expect("Should be valid");
    matcher.match_address("/footron").expect("Should match");

    // Character class
    // Character classes are sets or ranges of characters to match.
    // e.g. [a-z] will match any lower case alphabetic character. [abcd] will match the characters abcd.
    // They can be negated with '!', e.g. [!0-9] will match all characters except 0-9
    // Basic example
    matcher = Matcher::new("/oscillator/[0-9]").expect("Should be valid");
    matcher.match_address("/oscillator/0").expect("Should match");  // Beginning of range included
    matcher.match_address("/oscillator/6").expect("Should match");  // Middle of range
    matcher.match_address("/oscillator/9").expect("Should match");  // Last member of range included

    // Inverted order should work too
    matcher = Matcher::new("/oscillator/[9-0]").expect("Should be valid");
    matcher.match_address("/oscillator/0").expect("Should match");
    matcher.match_address("/oscillator/6").expect("Should match");
    matcher.match_address("/oscillator/9").expect("Should match");

    // Multiple ranges
    matcher = Matcher::new("/oscillator/[a-zA-Z0-9]").expect("Should be valid");
    matcher.match_address("/oscillator/0").expect("Should match");
    matcher.match_address("/oscillator/a").expect("Should match");
    matcher.match_address("/oscillator/A").expect("Should match");

    // Negated range
    matcher = Matcher::new("/oscillator/[!0-9]").expect("Should be valid");
    matcher.match_address("/oscillator/1").expect_err("Should not match");
    matcher.match_address("/oscillator/a").expect("Should match");

    // Extra exclamation points must be entirely ignored
    matcher = Matcher::new("/oscillator/[!0-9!a-z!]").expect("Should be valid");
    matcher.match_address("/oscillator/A").expect("Should match");

    // Trailing dash has no special meaning
    matcher = Matcher::new("/oscillator/[abcd-]").expect("Should be valid");
    matcher.match_address("/oscillator/a").expect("Should match");
    matcher.match_address("/oscillator/-").expect("Should match");

    // Single wildcard
    // A single wildcard '?' matches excatly one alphanumeric character
    matcher = Matcher::new("/oscillator/?/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/1/frequency").expect("Should match");
    matcher.match_address("/oscillator/F/frequency").expect("Should match");
    matcher.match_address("/oscillator//frequency").expect_err("Should not match");
    matcher.match_address("/oscillator/10/frequency").expect_err("Should not match");

    // Test if two consecutive wildcards match
    matcher = Matcher::new("/oscillator/??/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/10/frequency").expect("Should match");
    matcher.match_address("/oscillator/1/frequency").expect_err("Should not match");

    // Test if it works if it is surrounded by non-wildcards
    matcher = Matcher::new("/oscillator/prefixed?postfixed/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/prefixed1postfixed/frequency").expect("Should match");
    matcher.match_address("/oscillator/prefixedpostfixed/frequency").expect_err("Should not match");

    // Wildcard
    // Wildcards '*' match zero or more alphanumeric characters. The implementation is greedy,
    // meaning it will match the longest possible sequence
    matcher = Matcher::new("/oscillator/*/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/anything123/frequency").expect("Should match");
    // Test that wildcard doesn't cross part boundary
    matcher.match_address("/oscillator/extra/part/frequency").expect_err("Should not match");
    matcher.match_address("/oscillator//frequency").expect_err("Should not match");

    // Test greediness
    matcher = Matcher::new("/oscillator/*bar/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/foobar/frequency").expect("Should match");
    matcher.match_address("/oscillator/foobarbar/frequency").expect("Should match");

    // Minimum length of 2
    matcher = Matcher::new("/oscillator/*??/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/foobar/frequency").expect("Should match");
    matcher.match_address("/oscillator/f/frequency").expect_err("Should not match");

    // Minimum length of 2 and another component follows
    matcher = Matcher::new("/oscillator/*??baz/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/foobarbaz/frequency").expect("Should match");
    matcher.match_address("/oscillator/fbaz/frequency").expect_err("Should not match");

    // Mix with character class
    matcher = Matcher::new("/oscillator/*[a-d]/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/a/frequency").expect("Should match");
    matcher.match_address("/oscillator/fooa/frequency").expect("Should match");
    matcher.match_address("/oscillator/foox/frequency").expect_err("Should not match");

    // Mix with choice
    matcher = Matcher::new("/oscillator/*{bar,baz}/frequency").expect("Should be valid");
    matcher.match_address("/oscillator/foobar/frequency").expect("Should match");
    matcher.match_address("/oscillator/baz/frequency").expect("Should match");
    matcher.match_address("/oscillator/something/frequency").expect_err("Should not match");

    // Check for allowed literal characters
    matcher = Matcher::new("/!\"$%&'()+-./0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~").expect("Should be valid");
    matcher.match_address("/!\"$%&'()+-./0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~").expect("Should match");
}

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
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
