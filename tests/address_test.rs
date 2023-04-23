extern crate rosc;

#[cfg(feature = "std")]
use rosc::address::{verify_address, verify_address_pattern, Matcher, OscAddress};

#[cfg(feature = "std")]
#[test]
fn test_matcher() {
    let mut matcher;

    // Regular address using only alphanumeric parts
    matcher = Matcher::new("/oscillator/1/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/1/frequency")).expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/1/phase")).expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/1/frequencyfoo"))
            .expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/prefix/oscillator/1/frequency"))
            .expect("Valid address pattern")
    ));

    // Choice
    matcher = Matcher::new("/foo{bar,baz}").expect("Should be valid");
    assert!(matcher
        .match_address(&OscAddress::new(String::from("/foobar")).expect("Valid address pattern")));
    assert!(matcher
        .match_address(&OscAddress::new(String::from("/foobaz")).expect("Valid address pattern")));

    matcher = Matcher::new("/foo{bar,baz,tron}").expect("Should be valid");
    assert!(matcher
        .match_address(&OscAddress::new(String::from("/footron")).expect("Valid address pattern")));

    // Character class
    // Character classes are sets or ranges of characters to match.
    // e.g. [a-z] will match any lower case alphabetic character. [abcd] will match the characters abcd.
    // They can be negated with '!', e.g. [!0-9] will match all characters except 0-9
    // Basic example
    matcher = Matcher::new("/oscillator/[0-9]").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/0")).expect("Valid address pattern")
    )); // Beginning of range included
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/6")).expect("Valid address pattern")
    )); // Middle of range
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/9")).expect("Valid address pattern")
    )); // Last member of range included

    // Inverted order should fail
    Matcher::new("/oscillator/[9-0]").expect_err("Inverted range accepted");

    // Multiple ranges
    matcher = Matcher::new("/oscillator/[a-zA-Z0-9]").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/0")).expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/a")).expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/A")).expect("Valid address pattern")
    ));

    // Negated range
    matcher = Matcher::new("/oscillator/[!0-9]").expect("Should be valid");
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/1")).expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/a")).expect("Valid address pattern")
    ));

    // Extra exclamation points must be entirely ignored
    matcher = Matcher::new("/oscillator/[!0-9!a-z!]").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/A")).expect("Valid address pattern")
    ));

    // Trailing dash has no special meaning
    matcher = Matcher::new("/oscillator/[abcd-]").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/a")).expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/-")).expect("Valid address pattern")
    ));

    // Single wildcard
    // A single wildcard '?' matches excatly one alphanumeric character
    matcher = Matcher::new("/oscillator/?/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/1/frequency")).expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/F/frequency")).expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/10/frequency")).expect("Valid address pattern")
    ));

    // Test if two consecutive wildcards match
    matcher = Matcher::new("/oscillator/??/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/10/frequency")).expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/1/frequency")).expect("Valid address pattern")
    ));

    // Test if it works if it is surrounded by non-wildcards
    matcher = Matcher::new("/oscillator/prefixed?postfixed/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/prefixed1postfixed/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/prefixedpostfixed/frequency"))
            .expect("Valid address pattern")
    ));

    // Wildcard
    // Wildcards '*' match zero or more alphanumeric characters. The implementation is greedy,
    // meaning it will match the longest possible sequence
    matcher = Matcher::new("/oscillator/*/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/anything123/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(matcher.match_address(&OscAddress::new(String::from("/oscillator/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~/frequency")).expect("Valid address pattern")));
    // Test that wildcard doesn't cross part boundary
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/extra/part/frequency"))
            .expect("Valid address pattern")
    ));

    // Test greediness
    matcher = Matcher::new("/oscillator/*bar/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobar/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobarbar/frequency"))
            .expect("Valid address pattern")
    ));

    // Minimum length of 2
    matcher = Matcher::new("/oscillator/*??/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobar/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/f/frequency")).expect("Valid address pattern")
    ));

    // Minimum length of 2 and another component follows
    matcher = Matcher::new("/oscillator/*??baz/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobarbaz/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/fbaz/frequency"))
            .expect("Valid address pattern")
    ));

    // Mix with character class
    matcher = Matcher::new("/oscillator/*[a-d]/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/a/frequency")).expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/fooa/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foox/frequency"))
            .expect("Valid address pattern")
    ));

    // Mix with choice
    matcher = Matcher::new("/oscillator/*{bar,baz}/frequency").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobar/frequency"))
            .expect("Valid address pattern")
    ));
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/baz/frequency")).expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/something/frequency"))
            .expect("Valid address pattern")
    ));

    // Wildcard as last part
    matcher = Matcher::new("/oscillator/*").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobar")).expect("Valid address pattern")
    ));
    assert!(!matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobar/frequency"))
            .expect("Valid address pattern")
    ));

    // Wildcard with more components in part but it's the last part
    matcher = Matcher::new("/oscillator/*bar").expect("Should be valid");
    assert!(matcher.match_address(
        &OscAddress::new(String::from("/oscillator/foobar")).expect("Valid address pattern")
    ));

    // Check for allowed literal characters
    matcher = Matcher::new(
        "/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~",
    )
    .expect("Should be valid");
    assert!(matcher.match_address(&OscAddress::new(String::from("/!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~")).expect("Valid address pattern")));

    // Check that single wildcard matches all legal characters
    matcher = Matcher::new("/?").expect("Should be valid");
    let legal =
        "!\"$%&'()+-.0123456789:;<=>@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_`abcdefghijklmnopqrstuvwxyz|~";
    for c in legal.chars() {
        assert!(matcher
            .match_address(&OscAddress::new(format!("/{}", c)).expect("Valid address pattern")));
    }

    // Make sure the character class deduplicator is triggered for code coverage
    matcher = Matcher::new("/[a-za-za-z]").expect("Should be valid");
    assert!(
        matcher.match_address(&OscAddress::new(String::from("/a")).expect("Valid address pattern"))
    );
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
    verify_address_pattern("/test[a-defgh]").expect("Should be valid");
    verify_address_pattern("/test[a-defg-z]").expect("Should be valid");
    verify_address_pattern("/test[a-za-z]").expect("Should be valid");
    verify_address_pattern("/test[a-z]*??/{foo,bar,baz}[!a-z0-9]/*").expect("Should be valid");
    verify_address_pattern("/test{foo}").expect("Should be valid");

    // Empty element in choice
    verify_address_pattern("/{asd,}/").expect_err("Should not be valid");
    // Illegal character in range
    verify_address_pattern("/[a-b*]/").expect_err("Should not be valid");
    // Character range is reversed
    verify_address_pattern("/[b-a]").expect_err("Should not be valid");
    // Character range starting and ending at same character
    verify_address_pattern("/[a-a]").expect_err("Should not be valid");

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
