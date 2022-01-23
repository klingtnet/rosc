use crate::errors::OscError;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_till1, take, is_a};
use nom::character::complete::{alphanumeric1, char, satisfy};
use nom::combinator::{all_consuming, complete, map_parser, verify};
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair};
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
        let pattern_parts = match all_consuming(many1(map_address_pattern_component))(pattern) {
            Ok((_, parts)) => { parts }
            Err(_) => panic!("Address must be valid")
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
        let mut remainder: &str = address;
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
                Err(_) => return Err(OscError::Unimplemented) // TODO
            };
        }
        return if remainder.len() == 0 {
            Ok(true)
        } else {
            Err(OscError::Unimplemented)     // TODO
        };
    }
}

fn map_alternative(s: &str) -> String {
    wrap_with(&s.replace(',', "|"), "(", ")")
}

fn wrap_with(s: &str, pre: &str, post: &str) -> String {
    pre.to_string() + s + post
}

fn map_wildcard(_: &str) -> String {
    r"\w*".into()
}

fn map_question_mark(_: &str) -> String {
    r"\w?".into()
}

fn parse_address_part(input: &str) -> IResult<&str, &str> {
    preceded(char('/'), take_till1(|c| " \t\r\n#*,/?[]{}".contains(c)))(input)
}

fn parse_address_pattern_part(input: &str) -> IResult<&str, &str> {
    preceded(char('/'), take_till1(|c| " \n\t\r/".contains(c)))(input)
}

// Translate OSC pattern rules into an regular expression.
// A pattern part can contain more than one rule, e.g. `{voice,synth}-[1-9]` contains two rules, an alternative and a number range.
fn parse_pattern_part(input: &str) -> IResult<&str, String> {
    many1(alt((
        delimited(char('{'), is_not("}"), char('}')).map(map_alternative),
        delimited(tag("[!"), is_not("]"), char(']')).map(|s: &str| wrap_with(s, "[^", "]")),
        delimited(char('['), is_not("]"), char(']')).map(|s: &str| wrap_with(s, "[", "]")),
        tag("*").map(map_wildcard),
        tag("?").map(map_question_mark),
        is_not("[{").map(|s: &str| s.to_owned()),
    )))(input)
    .map(|(input, parts)| (input, parts.concat()))
}

fn parse_address_pattern(input: &str) -> Result<Vec<Regex>, OscError> {
    let (_, patterns) = all_consuming(many1(map_parser(
        parse_address_pattern_part,
        parse_pattern_part,
    )))(input)
    .map_err(|_| OscError::BadAddressPattern("bad address pattern".to_string()))?;
    patterns
        .iter()
        .map(|p| Regex::new(p))
        .collect::<Result<Vec<Regex>, regex::Error>>()
        .map_err(|err| OscError::RegexError(err.to_string()))
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
        out.push(c as char);
    }
    // TODO: for funky but legal ranges like '0-a' we need to clean up the ranges
    out
}

/// Check whether a character is an allowed address character
/// All printable ASCII characters except for a few special characters are allowed
fn is_address_character(x: char) -> bool {
    match x {
        ' ' | '#' | '*' | ',' | '/' | '?' | '[' | ']' | '{' | '}' => false,
        c => c > '\x20' && c < '\x7F'
    }
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
            // TODO: Deduplicate?
            Ok((_, o)) => CharacterClass { negated, characters: o.concat() },
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
        alphanumeric1.map(|s: &str| { AddressPatternComponent::Tag(String::from(s)) }), // TODO: this should be all printable ascii characters, not just alphanum!
        // Slashes must be seperated into their own tag for the non-greedy implementation of wildcards
        char('/').map(|c: char| { AddressPatternComponent::Tag(c.to_string()) }),
        tag("?").map(|_| { AddressPatternComponent::WildcardSingle }),
        // Combinations of wildcards are a bit tricky.
        // Multiple '*' wildcards in a row are equal to a single '*'.
        // A '*' wildcard followed by any number of '?' wildcards is also equal to '*' but must
        // match at least the same amount of characters as there are '?' wildcards in the combination.
        // For example, '*??' must match at least 2 characters.
        is_a("*?").map(|x: &str| { AddressPatternComponent::Wildcard(x.matches("?").count()) }),
        map_parser(
            delimited(char('{'), is_not("}"), char('}')),
            separated_list0(char(','.into()), alphanumeric1),
        ).map(|alternatives: Vec<&str>| { AddressPatternComponent::Choice(alternatives.iter().map(|x| x.to_string()).collect()) }),
        delimited(char('['), is_not("]") /*TODO: Only allowed: alphanum and '!-' */, char(']')).map(|s: &str| { AddressPatternComponent::CharacterClass(CharacterClass::new(s)) })
    ))(input)
}

fn match_literally<'a>(input: &'a str, pattern: &str) -> IResult<&'a str, &'a str>
{
    tag(pattern)(input)
}

fn match_wildcard_single(input: &str) -> IResult<&str, &str>
{
    // TODO: this has to be possible with a simpler parser?
    verify(take(1usize), |s: &str| s.chars().all(char::is_alphanumeric))(input)
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
        // No next component, consume all alphanumeric characters until end or next '/'
        None => verify(alphanumeric1, |s: &str| s.len() >= minimum_length)(input),
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