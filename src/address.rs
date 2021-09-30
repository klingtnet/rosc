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
    for char in chars
    {
        if ((char as u8) < 0x20) | ((char as u8) > 0x7E) {
            return Err(OscError::BadAddress("Address may only contain printable ASCII characters"));
        }
        if " #*,?[]{}".chars().any(|x| char == x) {
            return Err(OscError::BadAddress("Address may not contain any of the following characters: ' #*,?[]{}'"));
        }
    }

    return Ok(());
}

/// Check if the address of an OSC message is valid
pub fn validate_message_address(addr: &String) -> Result<()>
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

    // Validate rest of address
    let mut in_character_range = false;
    let mut in_string_list = false;
    for char in chars
    {
        if ((char as u8) < 0x20) | ((char as u8) > 0x7E) {
            return Err(OscError::BadAddress("Address may only contain printable ASCII characters"));
        }
        if " #".chars().any(|x| char == x) {
            return Err(OscError::BadAddress("Address may not contain any of the following characters: ' #'"));
        }
        if !in_string_list && char == ',' {
            return Err(OscError::BadAddress("Address may not contain any of the following characters outside of string lists: ','"));
        }
        if char == '[' {
            in_character_range = true;
            if in_string_list {
                return Err(OscError::BadAddress("Can not start a character range (square brackets) within string list (curly brackets)"));
            }
        }
        if (char == '/') & in_character_range {
            return Err(OscError::BadAddress("Character range (square brackets) was started but not closed before the next address part started"));
        }
        if char == ']' {
            in_character_range = false;
        }
        if char == '{' {
            in_string_list = true;
        }
        if (char == '/') & in_string_list {
            return Err(OscError::BadAddress("String list (curly brackets) was started but not closed before the next address part started"));
        }
        if char == '}' {
            in_string_list = false;
            if in_character_range {
                return Err(OscError::BadAddress("Can not start a string list (curly brackets) within character range (square brackets)"));
            }
        }
    }
    if in_character_range {
        return Err(OscError::BadAddress("Character range (square brackets) was started but not closed before the end of the address"));
    }
    if in_string_list {
        return Err(OscError::BadAddress("String list (curly brackets) was started but not closed before the end of the address"));
    }

    return Ok(());
}