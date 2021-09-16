use crate::errors::OscError;
use crate::types::{
    Result
};

/// Check if the address of an OSC method is valid
pub fn validate_method_address(addr: &String) -> Result<()>
{
    if !addr.is_ascii() {
        return Err(OscError::BadAddress("Address must only contain ASCII characters"));
    }
    if addr.len() < 1 {
        return Err(OscError::BadAddress("Address must be at least 1 character long"));
    }
    let mut chars = addr.chars();

    // Check if address starts with '/'
    let first = chars.next().unwrap();
    if first != '/' {
        return Err(OscError::BadAddress("Address must start with '/'"));
    }

    // Check if address contains illegal characters
    if chars.any(|x| " #*,?[]{}".chars().any(|y| y == x)) {
        return Err(OscError::BadAddress("Address may not contain any of the following characters: ' #*,?[]{}'"));
    }

    // TODO: Check if non-printable ascii characters are contained?

    return Ok(());
}