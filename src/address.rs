use crate::errors::OscError;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, take, take_while1, take_while_m_n};
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, complete, opt, recognize, verify};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, pair, separated_pair};
use nom::{IResult, Parser};
use std::collections::HashSet;
use std::iter::FromIterator;

/// A valid OSC method address.
///
/// A valid OSC address begins with a `/` and contains at least a method name, e.g. `/tempo`.
/// A plain address must not include any of the following characters `#*,/?[]{}`, since they're reserved for OSC address patterns.
#[derive(Clone, Debug)]
pub struct OscAddress(String);

impl OscAddress {
    pub fn new(address: String) -> Result<Self, OscError> {
        match verify_address(&address) {
            Ok(_) => Ok(OscAddress(address)),
            Err(e) => Err(e),
        }
    }
}

impl Display for OscAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// With a Matcher OSC method addresses can be [matched](Matcher::match_address) against an OSC address pattern.
/// Refer to the OSC specification for details about OSC address spaces: <http://opensoundcontrol.org/spec-1_0.html#osc-address-spaces-and-osc-addresses>
#[derive(Clone, Debug)]
pub struct Matcher {
    pub pattern: String,
    pattern_parts: Vec<AddressPatternComponent>,
}

impl Matcher {
    /// Instantiates a new `Matcher` with the given address pattern.
    /// An error will be returned if the given address pattern is invalid.
    ///
    /// Matcher should be instantiated once per pattern and reused because its construction requires parsing the address pattern which is computationally expensive.
    ///
    /// A valid address pattern begins with a `/` and contains at least a method name, e.g. `/tempo`.
    /// OSC defines a couple of rules that look like regular expression but are subtly different:
    ///
    /// - `?` matches a single character
    /// - `*` matches zero or more characters
    /// - `[a-z]` are basically regex [character classes](https://www.regular-expressions.info/charclass.html)
    /// - `{foo,bar}` is an alternative, matching either `foo` or `bar`
    /// - everything else is matched literally
    ///
    /// Refer to the OSC specification for details about address pattern matching: <https://opensoundcontrol.stanford.edu/spec-1_0.html#osc-message-dispatching-and-pattern-matching>.
    ///
    /// # Examples
    ///
    /// ```
    /// use rosc::address::Matcher;
    ///
    /// Matcher::new("/tempo").expect("valid address");
    /// Matcher::new("").expect_err("address does not start with a slash");
    /// ```
    pub fn new(pattern: &str) -> Result<Self, OscError> {
        verify_address_pattern(pattern)?;
        let mut match_fn = all_consuming(many1(map_address_pattern_component));
        let (_, pattern_parts) =
            match_fn(pattern).map_err(|err| OscError::BadAddressPattern(err.to_string()))?;

        Ok(Matcher {
            pattern: pattern.into(),
            pattern_parts,
        })
    }

    /// Match an OSC address against an address pattern.
    /// If the address matches the pattern the result will be `true`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rosc::address::{Matcher, OscAddress};
    ///
    /// let matcher = Matcher::new("/oscillator/[0-9]/{frequency,phase}").unwrap();
    /// assert!(matcher.match_address(&OscAddress::new(String::from("/oscillator/1/frequency")).unwrap()));
    /// assert!(matcher.match_address(&OscAddress::new(String::from("/oscillator/8/phase")).unwrap()));
    /// assert_eq!(matcher.match_address(&OscAddress::new(String::from("/oscillator/4/detune")).unwrap()), false);
    /// ```
    pub fn match_address(&self, address: &OscAddress) -> bool {
        // Trivial case
        if address.0 == self.pattern {
            return true;
        }

        let mut remainder = address.0.as_str();
        let mut iter = self.pattern_parts.iter().peekable();

        while let Some(part) = iter.next() {
            // Match the the address component by component
            let result = match part {
                AddressPatternComponent::Tag(s) => match_literally(remainder, s),
                AddressPatternComponent::WildcardSingle => match_wildcard_single(remainder),
                AddressPatternComponent::Wildcard(l) => {
                    match_wildcard(remainder, *l, iter.peek().copied())
                }
                AddressPatternComponent::CharacterClass(cc) => match_character_class(remainder, cc),
                AddressPatternComponent::Choice(s) => match_choice(remainder, s),
            };

            remainder = match result {
                Ok((i, _)) => i,
                Err(_) => return false, // Component didn't match, goodbye
            };
        }

        // Address is only matched if it was consumed entirely
        remainder.is_empty()
    }
}

/// Check whether a character is an allowed address character
/// All printable ASCII characters except for a few special characters are allowed
fn is_address_character(x: char) -> bool {
    if !x.is_ascii() || x.is_ascii_control() {
        return false;
    }

    ![' ', '#', '*', ',', '/', '?', '[', ']', '{', '}'].contains(&x)
}

/// Parser to turn a choice like '{foo,bar}' into a vector containing the choices, like ["foo", "bar"]
fn pattern_choice(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(
        char('{'),
        separated_list1(tag(","), take_while1(is_address_character)),
        char('}'),
    )(input)
}

/// Parser to recognize a character class like [!a-zA-Z] and return '!a-zA-Z'
fn pattern_character_class(input: &str) -> IResult<&str, &str> {
    let inner = pair(
        // It is important to read the leading negating '!' to make sure the rest of the parsed
        // character class isn't empty. E.g. '[!]' is not a valid character class.
        recognize(opt(tag("!"))),
        // Read all remaining character ranges and single characters
        // We also need to verify that ranges are increasing by ASCII value. For example, a-z is
        // valid, but z-a or a-a is not.
        recognize(many1(verify(
            alt((
                separated_pair(
                    satisfy(is_address_character),
                    char('-'),
                    satisfy(is_address_character),
                ),
                // Need to map this into a tuple to make it compatible with the output of the
                // separated pair parser above. Will always validate as true.
                satisfy(is_address_character).map(|c| ('\0', c)),
            )),
            |(o1, o2): &(char, char)| o1 < o2,
        ))),
    );

    delimited(char('['), recognize(inner), char(']'))(input)
}

/// A characters class is defined by a set or range of characters that it matches.
/// For example, [a-z] matches all lowercase alphabetic letters. It can also contain multiple
/// ranges, like [a-zA-Z]. Instead of a range you can also directly provide the characters to match,
/// e.g. [abc123]. You can also combine this with ranges, like [a-z123].
/// If the first characters is an exclamation point, the match is negated, e.g. [!0-9] will match
/// anything except numbers.
#[derive(Clone, Debug)]
struct CharacterClass {
    pub negated: bool,
    pub characters: String,
}

/// Expand a character range like 'a-d' to all the letters contained in the range, e.g. 'abcd'
/// This is done by converting the characters to their ASCII values and then getting every ASCII
/// in between.
fn expand_character_range(first: char, second: char) -> String {
    let start = first as u8;
    let end = second as u8;

    let range = start..=end;

    range
        .into_iter()
        .map(char::from)
        .filter(|c| is_address_character(*c))
        .collect()
}

impl CharacterClass {
    pub fn new(s: &str) -> Self {
        let mut input = s;
        let negated;
        match char::<_, nom::error::Error<&str>>('!')(input) {
            Ok((i, _)) => {
                negated = true;
                input = i;
            }
            Err(_) => negated = false,
        }

        let characters = complete(many1(alt((
            // '!' besides at beginning has no special meaning, but is legal
            char::<_, nom::error::Error<&str>>('!').map(|_| String::from("")),
            // attempt to match a range like a-z or 0-9
            separated_pair(
                satisfy(is_address_character),
                char('-'),
                satisfy(is_address_character),
            )
            .map(|(first, second)| expand_character_range(first, second)),
            // Match characters literally
            satisfy(is_address_character).map(|x| x.to_string()),
            // Trailing dash
            char('-').map(|_| String::from("-")),
        ))))(input);

        match characters {
            Ok((_, o)) => CharacterClass {
                negated,
                characters: HashSet::<char>::from_iter(o.concat().chars())
                    .iter()
                    .collect(),
            },
            _ => {
                panic!("Invalid character class formatting {}", s)
            }
        }
    }
}

#[derive(Clone, Debug)]
enum AddressPatternComponent {
    Tag(String),
    Wildcard(usize),
    WildcardSingle,
    CharacterClass(CharacterClass),
    Choice(Vec<String>),
}

fn map_address_pattern_component(input: &str) -> IResult<&str, AddressPatternComponent> {
    alt((
        // Anything that's alphanumeric gets matched literally
        take_while1(is_address_character)
            .map(|s: &str| AddressPatternComponent::Tag(String::from(s))),
        // Slashes must be seperated into their own tag for the non-greedy implementation of wildcards
        char('/').map(|c: char| AddressPatternComponent::Tag(c.to_string())),
        tag("?").map(|_| AddressPatternComponent::WildcardSingle),
        // Combinations of wildcards are a bit tricky.
        // Multiple '*' wildcards in a row are equal to a single '*'.
        // A '*' wildcard followed by any number of '?' wildcards is also equal to '*' but must
        // match at least the same amount of characters as there are '?' wildcards in the combination.
        // For example, '*??' must match at least 2 characters.
        is_a("*?").map(|x: &str| AddressPatternComponent::Wildcard(x.matches('?').count())),
        pattern_choice.map(|choices: Vec<&str>| {
            AddressPatternComponent::Choice(choices.iter().map(|x| x.to_string()).collect())
        }),
        pattern_character_class
            .map(|s: &str| AddressPatternComponent::CharacterClass(CharacterClass::new(s))),
    ))(input)
}

fn match_literally<'a>(input: &'a str, pattern: &str) -> IResult<&'a str, &'a str> {
    tag(pattern)(input)
}

fn match_wildcard_single(input: &str) -> IResult<&str, &str> {
    take_while_m_n(1, 1, is_address_character)(input)
}

fn match_character_class<'a>(
    input: &'a str,
    character_class: &'a CharacterClass,
) -> IResult<&'a str, &'a str> {
    if character_class.negated {
        is_not(character_class.characters.as_str())(input)
    } else {
        is_a(character_class.characters.as_str())(input)
    }
}

/// Sequentially try all tags from choice element until one matches or return an error
/// Example choice element: '{foo,bar}'
/// It will get parsed into a vector containing the strings "foo" and "bar", which are then matched
fn match_choice<'a>(input: &'a str, choices: &[String]) -> IResult<&'a str, &'a str> {
    for choice in choices {
        if let Ok((i, o)) = tag::<_, _, nom::error::Error<&str>>(choice.as_str())(input) {
            return Ok((i, o));
        }
    }
    Err(nom::Err::Error(nom::error::Error::from_error_kind(
        input,
        ErrorKind::Tag,
    )))
}

/// Match Wildcard '*' by either consuming the rest of the part, or, if it's not the last component
/// in the part, by looking ahead and matching the next component
fn match_wildcard<'a>(
    input: &'a str,
    minimum_length: usize,
    next: Option<&AddressPatternComponent>,
) -> IResult<&'a str, &'a str> {
    // If the next component is a '/', there are no more components in the current part and it can be wholly consumed
    let next = next.filter(|&part| match part {
        AddressPatternComponent::Tag(s) => s != "/",
        _ => true,
    });
    match next {
        // No next component, consume all allowed characters until end or next '/'
        None => verify(take_while1(is_address_character), |s: &str| {
            s.len() >= minimum_length
        })(input),
        // There is another element in this part, so logic gets a bit more complicated
        Some(component) => {
            // Wildcards can only match within the current address part, discard the rest
            let address_part = match input.split_once('/') {
                Some((p, _)) => p,
                None => input,
            };

            // Attempt to find the latest matching occurrence of the next pattern component
            // This is a greedy wildcard implementation
            let mut longest: usize = 0;
            for i in 0..address_part.len() {
                let (_, substring) = input.split_at(i);
                let result: IResult<_, _, nom::error::Error<&str>> = match component {
                    AddressPatternComponent::Tag(s) => match_literally(substring, s.as_str()),
                    AddressPatternComponent::CharacterClass(cc) => {
                        match_character_class(substring, cc)
                    }
                    AddressPatternComponent::Choice(s) => match_choice(substring, s),
                    // These two cases are prevented from happening by map_address_pattern_component
                    AddressPatternComponent::WildcardSingle => {
                        panic!("Single wildcard ('?') must not follow wildcard ('*')")
                    }
                    AddressPatternComponent::Wildcard(_) => {
                        panic!("Double wildcards must be condensed into one")
                    }
                };

                if result.is_ok() {
                    longest = i
                }
            }
            verify(take(longest), |s: &str| s.len() >= minimum_length)(input)
        }
    }
}

/// Verify that an address is valid
///
/// # Examples
/// ```
/// use rosc::address::verify_address;
///
/// match verify_address("/oscillator/1") {
///     Ok(()) => println!("Address is valid"),
///     Err(e) => println!("Address is not valid")
/// }
/// ```
pub fn verify_address(input: &str) -> Result<(), OscError> {
    match all_consuming::<_, _, nom::error::Error<&str>, _>(many1(pair(
        tag("/"),
        take_while1(is_address_character),
    )))(input)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(OscError::BadAddress("Invalid address".to_string())),
    }
}

/// Parse an address pattern's part until the next '/' or the end
fn address_pattern_part_parser(input: &str) -> IResult<&str, Vec<&str>> {
    many1::<_, _, nom::error::Error<&str>, _>(alt((
        take_while1(is_address_character),
        tag("?"),
        tag("*"),
        recognize(pattern_choice),
        pattern_character_class,
    )))(input)
}

/// Verify that an address pattern is valid
///
/// # Examples
/// ```
/// use rosc::address::verify_address_pattern;
///
/// match verify_address_pattern("/oscillator/[0-9]/*") {
///     Ok(()) => println!("Address is valid"),
///     Err(e) => println!("Address is not valid")
/// }
/// ```
pub fn verify_address_pattern(input: &str) -> Result<(), OscError> {
    match all_consuming(many1(
        // Each part must start with a '/'. This automatically also prevents a trailing '/'
        pair(tag("/"), address_pattern_part_parser.map(|x| x.concat())),
    ))(input)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(OscError::BadAddress("Invalid address pattern".to_string())),
    }
}
