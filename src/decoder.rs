use crate::NomResult;
use crate::errors::OscError;
use crate::types::{
    OscArray,
    OscBundle,
    OscColor,
    OscMessage,
    OscMidiMessage,
    OscPacket,
    OscTime,
    OscType,
};

#[cfg(feature = "std")]
use std::io::{BufRead, Read};
#[cfg(feature = "std")]
use std::{char, io};
use crate::alloc::{
    string::{ String, ToString },
    vec::Vec,
};

use nom::sequence::terminated;
use nom::{Offset};
use nom::bytes::complete::{take, take_till};
use nom::combinator::{map, map_parser};
use nom::multi::many0;
use nom::number::complete::{be_f32, be_f64, be_i32, be_i64, be_u32};
use nom::{
    IResult,
    combinator::map_res,
    sequence::tuple
};

/// Common MTU size for ethernet
pub const MTU: usize = 1536;

/// Takes an byte slice as argument and returns an
/// OSC packet on success or an `OscError` if the slice
/// does not contain a valid OSC message.
pub fn decode(msg: &[u8]) -> NomResult<OscPacket> {
    decode_packet(msg, msg).map(|(_, packet)| packet)
}

fn decode_packet<'a>(input: &'a [u8], original_input: &'a [u8]) -> IResult<&'a [u8], OscPacket, OscError> {
    if input.is_empty() {
        return Err(nom::Err::Error(OscError::BadPacket("Empty packet.")));
    }

    match input[0] as char {
        '/' => decode_message(input, original_input),
        '#' => decode_bundle(input, original_input),
        _ => Err(nom::Err::Error(OscError::BadPacket("Unknown message format."))),
    }
}

fn decode_message<'a>(input: &'a [u8], original_input: &'a [u8]) -> IResult<&'a [u8], OscPacket, OscError> {
    let (input, addr) = read_osc_string(input, original_input)?;
    let (input, type_tags) = read_osc_string(input, original_input)?;

    if type_tags.len() > 1 {
        let (input, args) = read_osc_args(input, original_input, type_tags)?;
        Ok((input, OscPacket::Message(OscMessage { addr, args })))
    } else {
        Ok((input, OscPacket::Message(OscMessage { addr, args: vec![] })))
    }
}

fn decode_bundle<'a>(input: &'a [u8], original_input: &'a [u8]) -> IResult<&'a [u8], OscPacket, OscError> {
    let (input, bundle_tag) = read_osc_string(input, original_input)?;
    if bundle_tag != "#bundle" {
        return Err(nom::Err::Error(OscError::BadBundle(format!(
            "Wrong bundle specifier: {}",
            bundle_tag
        ))));
    }

    let (input, (timetag, content)) = tuple((
        read_time_tag,
        many0(|input| read_bundle_element(input, original_input)),
    ))(input)?;

    Ok((
        input,
        OscPacket::Bundle(OscBundle {
            timetag,
            content,
        }),
    ))
}

fn read_bundle_element<'a>(input: &'a [u8], original_input: &'a [u8]) -> IResult<&'a [u8], OscPacket, OscError> {
    let (input, elem_size) = be_u32(input)?;

    let result = map_parser(
        |input| {
            take(elem_size)(input).map_err(|_: nom::Err<OscError>| {
                nom::Err::Error(OscError::BadBundle(
                    "Bundle shorter than expected!".to_string(),
                ))
            })
        },
        |input| decode_packet(input, original_input),
    )(input);
    drop(elem_size);
    result
}

fn read_osc_string<'a>(input: &'a [u8], original_input: &'a [u8]) -> IResult<&'a [u8], String, OscError> {
    map_res(
        terminated(
            take_till(|c| c == b'0'),
            |input| pad_to_4_byte_boundary(input, original_input),
        ),
        |str_buf| {
            String::from_utf8(str_buf.into())
                .map_err(OscError::StringError)
                .map(|s| s.trim_matches(0u8 as char).to_string())
        },
    )(input)
}

fn read_osc_args<'a>(
    mut input: &'a[u8],
    original_input: &'a [u8],
    raw_type_tags: String,
) ->  IResult<&'a [u8], Vec<OscType>, OscError> {
    let type_tags: Vec<char> = raw_type_tags.chars().skip(1).collect();

    let mut args: Vec<OscType> = Vec::with_capacity(type_tags.len());
    let mut stack: Vec<Vec<OscType>> = Vec::new();
    for tag in type_tags {
        if tag == '[' {
            // array start: save current frame and start a new frame
            // for the array's content
            stack.push(args);
            args = Vec::new();
        } else if tag == ']' {
            // found the end of the current array:
            // create array object from current frame and step one level up
            let array = OscType::Array(OscArray { content: args });
            match stack.pop() {
                Some(stashed) => args = stashed,
                None => return Err(nom::Err::Error(OscError::BadMessage(
                    "Encountered ] outside array"
                ))),
            }
            args.push(array);
        } else {
            let input_and_arg = read_osc_arg(input, original_input, tag)?;
            input = input_and_arg.0;
            args.push(input_and_arg.1);
        }
    }
    Ok((input, args))
}

fn read_osc_arg<'a>(input: &'a[u8], original_input: &'a [u8], tag: char) -> IResult<&'a[u8], OscType, OscError> {
    match tag {
        'f' => map(be_f32, OscType::Float)(input),
        'd' => map(be_f64, OscType::Double)(input),
        'i' => map(be_i32, OscType::Int)(input),
        'h' => map(be_i64, OscType::Long)(input),
        's' => {
            read_osc_string(input, original_input)
                .map(|(remainder, string)| (remainder, OscType::String(string)))
        }
        't' => read_time_tag(input).map(|(remainder, time)| {
            (remainder, OscType::Time(time))
        }),
        'b' => read_blob(input, original_input),
        'r' => read_osc_color(input),
        'T' => Ok((input, true.into())),
        'F' => Ok((input, false.into())),
        'N' => Ok((input, OscType::Nil)),
        'I' => Ok((input, OscType::Inf)),
        'c' => read_char(input),
        'm' => read_midi_message(input),
        _ => Err(nom::Err::Error(OscError::BadArg(format!(
            "Type tag \"{}\" is not implemented!",
            tag
        )))),
    }
}

fn read_char(input: &[u8]) -> IResult<&[u8], OscType, OscError> {
    map_res(
        be_u32,
        |b| {
            let opt_char = char::from_u32(b);
            match opt_char {
                Some(c) => Ok(OscType::Char(c)),
                None => Err(OscError::BadArg(
                    "Argument is not a char!".to_string(),
                )),
            }
        },
    )(input)
}

fn read_blob<'a>(input: &'a[u8], original_input: &'a [u8]) -> IResult<&'a[u8], OscType, OscError> {
    let (input, size) = be_u32(input)?;

    map(
        terminated(
            take(size),
            |input| pad_to_4_byte_boundary(input, original_input),
        ),
        |blob| OscType::Blob(blob.into()),
    )(input)
}

fn read_time_tag<'a>(input: &'a[u8]) -> IResult<&'a[u8], OscTime, OscError> {
    map(
        tuple((be_u32, be_u32)),
        |(seconds, fractional)| OscTime {
            seconds,
            fractional,
        },
    )(input)
}

fn read_midi_message<'a>(input: &'a[u8]) -> IResult<&'a[u8], OscType, OscError> {
    map(take(4usize), |buf: &[u8]| {
        OscType::Midi(OscMidiMessage {
            port: buf[0],
            status: buf[1],
            data1: buf[2],
            data2: buf[3],
        })
    })(input)
}

fn read_osc_color<'a>(input: &'a[u8]) -> IResult<&'a[u8], OscType, OscError> {
    map(take(4usize), |buf: &[u8]| {
        OscType::Color(OscColor {
            red: buf[0],
            green: buf[1],
            blue: buf[2],
            alpha: buf[3],
        })
    })(input)
}

fn pad_to_4_byte_boundary<'a>(input: &'a[u8], original_input: &'a[u8]) -> IResult<&'a[u8], (), OscError> {
    let offset = original_input.offset(input);
    let (input, _) = take(offset)(input)?;
    Ok((input, ()))
}
