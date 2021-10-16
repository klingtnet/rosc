use crate::errors::OscError;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::char;
use nom::combinator::{all_consuming, map_parser};
use nom::multi::many1;
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};
use regex::Regex;

/// With a Matcher OSC method addresses can be [matched](Matcher::match_address) against an OSC address pattern.
/// Refer to the OSC specification for details about OSC address spaces: <http://opensoundcontrol.org/spec-1_0.html#osc-address-spaces-and-osc-addresses>
pub struct Matcher {
    res: Vec<Regex>,
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
    /// - everything else is matches literally
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
        let res = parse_address_pattern(pattern)?;
        Ok(Matcher { res: res })
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
        let (_, parts) = all_consuming(many1(parse_address_part))(address)
            .map_err(|_| OscError::BadAddress("bad address".to_string()))?;
        if parts.len() != self.res.len() {
            return Ok(false);
        }
        Ok(self
            .res
            .iter()
            .zip(parts)
            .all(|(re, part)| re.is_match(part)))
    }
}

fn map_alternative(s: &str) -> String {
    wrap_with(&s.replace(',', "|"), "(", ")")
}

fn wrap_with(s: &str, pre: &str, post: &str) -> String {
    pre.to_string() + s + post.into()
}

fn map_wildcard(_: &str) -> String {
    r"\w*".into()
}

fn map_question_mark(_: &str) -> String {
    r"\w?".into()
}

fn parse_address_part(input: &str) -> IResult<&str, &str> {
    // TODO: the second parser, is_not, should require a non empty match
    preceded(char('/'), is_not(" \t\r\n#*,/?[]{}"))(input)
}

fn parse_address_pattern_part(input: &str) -> IResult<&str, &str> {
    // TODO: the second parser, is_not, should require a non empty match
    preceded(char('/'), is_not(" \n\t\r/"))(input)
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
