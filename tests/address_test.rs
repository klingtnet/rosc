extern crate rosc;

#[cfg(feature = "std")]
use rosc::address::{Matcher, verify_address};
use rosc::address::verify_address_pattern;

#[cfg(feature = "std")]
#[test]
fn test_matcher() {
    let mut matcher;

    // Regular address using only alphanumeric parts
    matcher = Matcher::new("/oscillator/1/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/1/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/1/phase").expect("Valid address pattern"), false);
    assert_eq!(matcher.match_address("/oscillator/1/frequencyfoo").expect("Valid address pattern"), false);
    assert_eq!(matcher.match_address("/prefix/oscillator/1/frequency").expect("Valid address pattern"), false);

    // Choice
    matcher = Matcher::new("/foo{bar,baz}").expect("Should be valid");
    assert_eq!(matcher.match_address("/foobar").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/foobaz").expect("Valid address pattern"), true);

    matcher = Matcher::new("/foo{bar,baz,tron}").expect("Should be valid");
    assert_eq!(matcher.match_address("/footron").expect("Valid address pattern"), true);

    // Character class
    // Character classes are sets or ranges of characters to match.
    // e.g. [a-z] will match any lower case alphabetic character. [abcd] will match the characters abcd.
    // They can be negated with '!', e.g. [!0-9] will match all characters except 0-9
    // Basic example
    matcher = Matcher::new("/oscillator/[0-9]").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/0").expect("Valid address pattern"), true);  // Beginning of range included
    assert_eq!(matcher.match_address("/oscillator/6").expect("Valid address pattern"), true);  // Middle of range
    assert_eq!(matcher.match_address("/oscillator/9").expect("Valid address pattern"), true);  // Last member of range included

    // Inverted order should work too
    matcher = Matcher::new("/oscillator/[9-0]").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/0").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/6").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/9").expect("Valid address pattern"), true);

    // Multiple ranges
    matcher = Matcher::new("/oscillator/[a-zA-Z0-9]").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/0").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/a").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/A").expect("Valid address pattern"), true);

    // Negated range
    matcher = Matcher::new("/oscillator/[!0-9]").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/1").expect("Valid address pattern"), false);
    assert_eq!(matcher.match_address("/oscillator/a").expect("Valid address pattern"), true);

    // Extra exclamation points must be entirely ignored
    matcher = Matcher::new("/oscillator/[!0-9!a-z!]").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/A").expect("Valid address pattern"), true);

    // Trailing dash has no special meaning
    matcher = Matcher::new("/oscillator/[abcd-]").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/a").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/-").expect("Valid address pattern"), true);

    // Single wildcard
    // A single wildcard '?' matches excatly one alphanumeric character
    matcher = Matcher::new("/oscillator/?/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/1/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/F/frequency").expect("Valid address pattern"), true);
    matcher.match_address("/oscillator//frequency").expect_err("Invalid address");
    assert_eq!(matcher.match_address("/oscillator/10/frequency").expect("Valid address pattern"), false);

    // Test if two consecutive wildcards match
    matcher = Matcher::new("/oscillator/??/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/10/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/1/frequency").expect("Valid address pattern"), false);

    // Test if it works if it is surrounded by non-wildcards
    matcher = Matcher::new("/oscillator/prefixed?postfixed/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/prefixed1postfixed/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/prefixedpostfixed/frequency").expect("Valid address pattern"), false);

    // Wildcard
    // Wildcards '*' match zero or more alphanumeric characters. The implementation is greedy,
    // meaning it will match the longest possible sequence
    matcher = Matcher::new("/oscillator/*/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/anything123/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~/frequency").expect("Valid address pattern"), true);
    // Test that wildcard doesn't cross part boundary
    assert_eq!(matcher.match_address("/oscillator/extra/part/frequency").expect("Valid address pattern"), false);
    matcher.match_address("/oscillator//frequency").expect_err("Invalid address");

    // Test greediness
    matcher = Matcher::new("/oscillator/*bar/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/foobar/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/foobarbar/frequency").expect("Valid address pattern"), true);

    // Minimum length of 2
    matcher = Matcher::new("/oscillator/*??/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/foobar/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/f/frequency").expect("Valid address pattern"), false);

    // Minimum length of 2 and another component follows
    matcher = Matcher::new("/oscillator/*??baz/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/foobarbaz/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/fbaz/frequency").expect("Valid address pattern"), false);

    // Mix with character class
    matcher = Matcher::new("/oscillator/*[a-d]/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/a/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/fooa/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/foox/frequency").expect("Valid address pattern"), false);

    // Mix with choice
    matcher = Matcher::new("/oscillator/*{bar,baz}/frequency").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/foobar/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/baz/frequency").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/something/frequency").expect("Valid address pattern"), false);

    // Wildcard as last part
    matcher = Matcher::new("/oscillator/*").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/foobar").expect("Valid address pattern"), true);
    assert_eq!(matcher.match_address("/oscillator/foobar/frequency").expect("Valid address pattern"), false);

    // Wildcard with more components in part but it's the last part
    matcher = Matcher::new("/oscillator/*bar").expect("Should be valid");
    assert_eq!(matcher.match_address("/oscillator/foobar").expect("Valid address pattern"), true);

    // Check for allowed literal characters
    matcher = Matcher::new("/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~").expect("Should be valid");
    assert_eq!(matcher.match_address("/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~").expect("Valid address pattern"), true);

    // Check that single wildcard matches all legal characters
    matcher = Matcher::new("/?").expect("Should be valid");
    let legal = "!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~";
    for c in legal.chars() {
        assert_eq!(matcher.match_address(format!("/{}", c).as_str()).expect("Valid address pattern"), true);
    }

    // Make sure the character class deduplicator is triggered for code coverage
    matcher = Matcher::new("/[a-za-za-z]").expect("Should be valid");
    assert_eq!(matcher.match_address("/a").expect("Valid address pattern"), true);
}

#[cfg(feature = "std")]
#[test]
fn test_verify_address() {
    verify_address("/test").expect("Should be valid");
    verify_address("/oscillator/1/frequency").expect("Should be valid");
    verify_address("/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~/foo").expect("Should be valid");

    // No '/' at beginning
    verify_address("test").expect_err("Should not be valid");
    // '/' at the end
    verify_address("/test/").expect_err("Should not be valid");
    // Different address pattern elements that are not allowed in regular addresses
    verify_address("/test*").expect_err("Should not be valid");
    verify_address("/test?").expect_err("Should not be valid");
    verify_address("/test{foo,bar}").expect_err("Should not be valid");
    verify_address("/test[a-z]").expect_err("Should not be valid");
}

#[cfg(feature = "std")]
#[test]
fn test_verify_address_pattern() {
    verify_address_pattern("/test").expect("Should be valid");
    verify_address_pattern("/oscillator/1/frequency").expect("Should be valid");
    verify_address_pattern("/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~/foo").expect("Should be valid");

    // No '/' at beginning
    verify_address_pattern("test").expect_err("Should not be valid");
    // '/' at the end
    verify_address_pattern("/test/").expect_err("Should not be valid");

    // Different address pattern elements
    verify_address_pattern("/test*").expect("Should be valid");
    verify_address_pattern("/test?").expect("Should be valid");
    verify_address_pattern("/test{foo,bar}").expect("Should be valid");
    verify_address_pattern("/test[a-z]").expect("Should be valid");
    verify_address_pattern("/test[a-za-z]").expect("Should be valid");
    verify_address_pattern("/test[a-z]*??/{foo,bar,baz}[!a-z0-9]/*").expect("Should be valid");
    verify_address_pattern("/test{foo}").expect("Should be valid");

    // Empty element in choice
    verify_address_pattern("/{asd,}/").expect_err("Should not be valid");
    // Illegal character in range
    verify_address_pattern("/[a-b*]/").expect_err("Should not be valid");

    // Empty
    verify_address_pattern("").expect_err("Should not be valid");
    // Empty part
    verify_address_pattern("//empty/part").expect_err("Should not be valid");
    // Unclosed range
    verify_address_pattern("/[a-/foo").expect_err("Should not be valid");
    verify_address_pattern("/[a-").expect_err("Should not be valid");
    // Empty range
    verify_address_pattern("/[]").expect_err("Should not be valid");
    verify_address_pattern("/[!]").expect_err("Should not be valid");
    // Unclosed alternative
    verify_address_pattern("/{foo,bar/foo").expect_err("Should not be valid");
    verify_address_pattern("/{foo,/bar").expect_err("Should not be valid");
    verify_address_pattern("/{foo").expect_err("Should not be valid");
    verify_address_pattern("/foo{,").expect_err("Should not be valid");
}
