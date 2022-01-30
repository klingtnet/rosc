use crate::errors::OscError;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_till1, take, is_a, take_while1};
use nom::character::complete::{char, satisfy};
use nom::combinator::{all_consuming, complete, map_parser, opt, recognize, verify};
use nom::multi::{many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::{IResult, Parser};
use nom::error::{ErrorKind, ParseError};
use regex::Regex;

/// With a Matcher OSC method addresses can be [matched](Matcher::match_address) against an OSC address pattern.
/// Refer to the OSC specification for details about OSC address spaces: <http://opensoundcontrol.org/spec-1_0.html#osc-address-spaces-and-osc-addresses>
#[derive(Debug)]
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
    /// Refer to the OSC specification for details about address pattern matching: <osc-message-dispatching-and-pattern-matching>.
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
        let pattern_parts = match all_consuming(many1(map_address_pattern_component))(pattern) {
            Ok((_, parts)) => { parts }
            // This should never happen because pattern is verified above
            Err(_) => panic!("Address pattern must be valid")
        };

        Ok(Matcher { pattern: String::from(pattern), pattern_parts })
    }

    /// Match an OSC address against an address pattern.
    /// If the address matches the pattern the result will be `true`, otherwise `false`.
    /// An error is returned if the given OSC address is not valid.
    ///
    /// A valid OSC address begins with a `/` and contains at least a method name, e.g. `/tempo`.
    /// Despite OSC address patterns a plain address must not include any of the following characters `#*,/?[]{}`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rosc::address::Matcher;
    ///
    /// let matcher = Matcher::new("/oscillator/[0-9]/{frequency,phase}").unwrap();
    /// assert!(matcher.match_address("/oscillator/1/frequency").unwrap());
    /// assert!(matcher.match_address("/oscillator/8/phase").unwrap());
    /// assert_eq!(matcher.match_address("/oscillator/4/detune").unwrap(), false);
    /// ```
    pub fn match_address(&self, address: &str) -> Result<bool, OscError> {
        verify_address(address)?;   // TODO: Create an address struct so we don't have to re-check addresses every time we match
        // Trivial case
        if address == self.pattern {
            return Ok(true);
        }
        let mut remainder: &str = address;
        // Match the the address component by component
        for (index, part) in self.pattern_parts.as_slice().iter().enumerate() {
            let result = match part {
                AddressPatternComponent::Tag(s) => match_literally(remainder, s.as_str()),
                AddressPatternComponent::WildcardSingle => match_wildcard_single(remainder),
                AddressPatternComponent::Wildcard(l) => {
                    // Check if this is the last pattern component
                    if index < self.pattern_parts.len() - 1 {
                        let next = &self.pattern_parts[index + 1];
                        match next {
                            // If the next component is a '/', there are no more components in the current part and it can be wholly consumed
                            AddressPatternComponent::Tag(s) if s == "/" => match_wildcard(remainder, l.clone(), None),
                            _ => match_wildcard(remainder, l.clone(), Some(next))
                        }
                    } else {
                        match_wildcard(remainder, l.clone(), None)
                    }
                }
                AddressPatternComponent::CharacterClass(cc) => match_character_class(remainder, cc),
                AddressPatternComponent::Choice(s) => match_choice(remainder, s),
            };

            match result {
                Ok((i, _)) => remainder = i,
                Err(_) => return Ok(false)  // Component didn't match, goodbye
            };
        }

        // Address is only matched if it was consumed entirely
        return if remainder.len() == 0 {
            Ok(true)
        } else {
            Ok(false)
        };
    }
}

/// Check whether a character is an allowed address character
/// All printable ASCII characters except for a few special characters are allowed
fn is_address_character(x: char) -> bool {
    match x {
        ' ' | '#' | '*' | ',' | '/' | '?' | '[' | ']' | '{' | '}' => false,
        c => c > '\x20' && c < '\x7F'
    }
}

/// Parser to turn a choice like '{foo,bar}' into a vector containing the choices, like ["foo", "bar"]
fn pattern_choice(input: &str) -> IResult<&str, Vec<&str>>
{
    delimited(char('{'), separated_list1(tag(","), take_while1(is_address_character)), char('}'))(input)
}

/// Parser to recognize a character class like [!a-zA-Z] and return '!a-zA-Z'
fn pattern_character_class(input: &str) -> IResult<&str, &str>
{
    let inner = pair(
        // It is important to read the leading negating '!' to make sure the rest of the parsed
        // character class isn't empty. E.g. '[!]' is not a valid character class.
        recognize(opt(tag("!"))),
        take_while1(is_address_character)
    );

    delimited(char('['), recognize(inner) , char(']'))(input)
}


/// A characters class is defined by a set or range of characters that it matches.
/// For example, [a-z] matches all lowercase alphabetic letters. It can also contain multiple
/// ranges, like [a-zA-Z]. Instead of a range you can also directly provide the characters to match,
/// e.g. [abc123]. You can also combine this with ranges, like [a-z123].
/// If the first characters is an exclamation point, the match is negated, e.g. [!0-9] will match
/// anything except numbers.
#[derive(Debug)]
struct CharacterClass {
    pub negated: bool,
    pub characters: String,
}

/// Expand a character range like 'a-d' to all the letters contained in the range, e.g. 'abcd'
/// This is done by converting the characters to their ASCII values and then getting every ASCII
/// in between.
fn expand_character_range<'a>(first: char, second: char) -> String {
    let start = first as u8;
    let end = second as u8;

    let range;
    if start >= end {
        range = end..=start;
    } else {
        range = start..=end;
    }

    let mut out = String::from("");
    for c in range {
        // For funky ranges like [0-a], some illegal characters are contained
        if is_address_character(c as char) {
            out.push(c as char);
        }
    }
    out
}

/// Removes all duplicates from a string of characters
fn deduplicate_characters(input: String) -> String
{
    let mut out: String = String::from("");
    for char in input.chars()
    {
        if !out.contains(char) {
            out.push(char);
        }
    }
    out
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
            Err(_) => negated = false
        }

        let characters = complete(many1(alt((
            // '!' besides at beginning has no special meaning, but is legal
            char::<_, nom::error::Error<&str>>('!').map(|_| String::from("")),
            // attempt to match a range like a-z or 0-9
            separated_pair(satisfy(is_address_character), char('-'), satisfy(is_address_character)).map(|(first, second)| expand_character_range(first, second)),
            // Match characters literally
            satisfy(is_address_character).map(|x| x.to_string()),
            // Trailing dash
            char('-').map(|_| { String::from("-") })
        ))))(input);

        match characters {
            Ok((_, o)) => CharacterClass { negated, characters: deduplicate_characters(o.concat()) },
            _ => { panic!("Invalid character class formatting {}", s) }
        }
    }
}

#[derive(Debug)]
enum AddressPatternComponent {
    Tag(String),
    Wildcard(usize),
    WildcardSingle,
    CharacterClass(CharacterClass),
    Choice(Vec<String>),
}

fn map_address_pattern_component(input: &str) -> IResult<&str, AddressPatternComponent>
{
    alt((
        // Anything that's alphanumeric gets matched literally
        take_while1(is_address_character).map(|s: &str| { AddressPatternComponent::Tag(String::from(s)) }),
        // Slashes must be seperated into their own tag for the non-greedy implementation of wildcards
        char('/').map(|c: char| { AddressPatternComponent::Tag(c.to_string()) }),
        tag("?").map(|_| { AddressPatternComponent::WildcardSingle }),
        // Combinations of wildcards are a bit tricky.
        // Multiple '*' wildcards in a row are equal to a single '*'.
        // A '*' wildcard followed by any number of '?' wildcards is also equal to '*' but must
        // match at least the same amount of characters as there are '?' wildcards in the combination.
        // For example, '*??' must match at least 2 characters.
        is_a("*?").map(|x: &str| { AddressPatternComponent::Wildcard(x.matches("?").count()) }),
        pattern_choice.map(|choices: Vec<&str>| { AddressPatternComponent::Choice(choices.iter().map(|x| x.to_string()).collect()) }),
        pattern_character_class.map(|s: &str| { AddressPatternComponent::CharacterClass(CharacterClass::new(s)) })
    ))(input)
}

fn match_literally<'a>(input: &'a str, pattern: &str) -> IResult<&'a str, &'a str>
{
    tag(pattern)(input)
}

fn match_wildcard_single(input: &str) -> IResult<&str, &str>
{
    // TODO: this has to be possible with a simpler parser?
    verify(take(1usize), |s: &str| s.chars().all(is_address_character))(input)
}

fn match_character_class<'a>(input: &'a str, character_class: &'a CharacterClass) -> IResult<&'a str, &'a str> {
    if character_class.negated {
        is_not(character_class.characters.as_str())(input)
    } else {
        is_a(character_class.characters.as_str())(input)
    }
}

/// Try all the tags in a choice element
/// Example choice element: '{foo,bar}'
/// It will get parsed into a vector containing the strings "foo" and "bar", which are then matched
fn match_choice<'a>(input: &'a str, choices: &Vec<String>) -> IResult<&'a str, &'a str> {
    for choice in choices
    {
        match tag::<_, _, nom::error::Error<&str>>(choice.as_str())(input) {
            Ok((i, o)) => return Ok((i, o)),
            Err(_) => {}
        }
    }
    return Err(nom::Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::Tag)));
}

/// Match Wildcard '*' by either consuming the rest of the part, or, if it's not the last component
/// in the part, by looking ahead and matching the next component
fn match_wildcard<'a>(input: &'a str, minimum_length: usize, next: Option<&AddressPatternComponent>) -> IResult<&'a str, &'a str>
{
    match next {
        // No next component, consume all allowed characters until end or next '/'
        None => verify(take_while1(is_address_character), |s: &str| s.len() >= minimum_length)(input),
        // There is another element in this part, so logic gets a bit more complicated
        Some(component) => {
            // Wildcards can only match within the current address part, discard the rest
            let address_part = match input.split_once("/") {
                Some((p, _)) => p,
                None => input
            };

            // Attempt to find the latest matching occurrence of the next pattern component
            // This is a greedy wildcard implementation
            let mut longest: usize = 0;
            for i in 0..address_part.len() {
                let (_, substring) = input.split_at(i);
                let result: IResult<_, _, nom::error::Error<&str>> = match component {
                    AddressPatternComponent::Tag(s) => match_literally(substring, s.as_str()),
                    AddressPatternComponent::CharacterClass(cc) => match_character_class(substring, cc),
                    AddressPatternComponent::Choice(s) => match_choice(substring, s),
                    // These two cases are prevented from happening by map_address_pattern_component
                    AddressPatternComponent::WildcardSingle => panic!("Single wildcard ('?') must not follow wildcard ('*')"),
                    AddressPatternComponent::Wildcard(_) => panic!("Double wildcards must be condensed into one"),
                };

                match result {
                    Ok(_) => longest = i,
                    _ => {}
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
pub fn verify_address(input: &str) -> Result<(), OscError>
{
    match all_consuming::<_,_,nom::error::Error<&str>,_>(
        many1(pair(tag("/"), take_while1(is_address_character)))
    )(input) {
        Ok((_)) => Ok(()),
        Err(_) => Err(OscError::BadAddress("Invalid address".to_string()))
    }
}

/// Parse an address pattern's part until the next '/' or the end
fn address_pattern_part_parser(input: &str) -> IResult<&str, Vec<&str>> {
    many1::<_, _, nom::error::Error<&str>, _>(
        alt((
            take_while1(is_address_character),
            tag("?"),
            tag("*"),
            recognize(pattern_choice),
            pattern_character_class,
        ))
    )(input)
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
pub fn verify_address_pattern(input: &str) -> Result<(), OscError>
{
    match all_consuming(many1(
        // Each part must start with a '/'. This automatically also prevents a trailing '/'
        pair(
            tag("/"),
            address_pattern_part_parser.map(|x| x.concat()),
        )
    ))(input)
    {
        Ok((_)) => Ok(()),
        Err(_) => Err(OscError::BadAddress("Invalid address pattern".to_string()))
    }
}