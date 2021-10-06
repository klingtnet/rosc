use crate::errors::OscError;
use crate::regex::Regex;
use crate::types::Result;

fn validate_address(addr: &str) -> Result<()> {
    if !addr.is_ascii() {
        return Err(OscError::BadAddress(
            "Address must only contain ASCII characters",
        ));
    }
    if addr.is_empty() {
        return Err(OscError::BadAddress(
            "Address must be at least 1 character long",
        ));
    }

    // Check if address starts with '/'
    if !addr.starts_with('/') {
        return Err(OscError::BadAddress("Address must start with '/'"));
    }

    // Check if address ends with '/'
    if addr.ends_with('/') {
        return Err(OscError::BadAddress("Address must not end with '/'"));
    }

    Ok(())
}

/// Check if the address of an OSC method is valid
pub fn validate_method_address(addr: &str) -> Result<()> {
    match validate_address(addr) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let chars = addr.chars();

    // Check if address contains illegal characters
    for char in chars {
        match char {
            _c if ((char as u8) < 0x20) | ((char as u8) > 0x7E) => {
                return Err(OscError::BadAddress(
                    "Address may only contain printable ASCII characters",
                ));
            }
            _c if " #*,?[]{}".chars().any(|x| char == x) => {
                return Err(OscError::BadAddress(
                    "Address may not contain any of the following characters: ' #*,?[]{}'",
                ));
            }
            _ => (),
        }
    }

    Ok(())
}

/// Check if the address of an OSC message is valid
pub fn validate_message_address(addr: &str) -> Result<()> {
    match validate_address(addr) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let chars = addr.chars();

    // Validate rest of address
    let mut in_character_range = false;
    let mut in_string_list = false;
    for char in chars {
        if ((char as u8) < 0x20) | ((char as u8) > 0x7E) {
            return Err(OscError::BadAddress(
                "Address may only contain printable ASCII characters",
            ));
        }
        if " #".chars().any(|x| char == x) {
            return Err(OscError::BadAddress(
                "Address may not contain any of the following characters: ' #'",
            ));
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
        return Err(OscError::BadAddress(
            "String list (curly brackets) was started but not closed before the end of the address",
        ));
    }

    Ok(())
}

/// Match a single part of two OSC addresses
fn match_part(message_part: &str, method_part: &str) -> bool {
    // direct match
    if message_part == method_part {
        return true;
    }

    // Use regex for everything else
    let mut pattern = message_part
        .to_string()
        .replace("*", "\\w*")
        .replace("?", "\\w")
        .replace("[!", "[^")
        .replace(",", ")|(?:")
        .replace("{", "(?:")
        .replace("}", ")");

    // Anchor the pattern to beginning and end of string
    pattern = format!("^{}$", pattern);

    let re = Regex::new(pattern.as_str()).unwrap();

    re.is_match(method_part)
}

/// Check if a message address matches a method address
pub fn match_address(message_addr: &str, method_addr: &str) -> Result<bool> {
    match validate_message_address(message_addr) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }
    match validate_method_address(method_addr) {
        Ok(()) => {}
        Err(e) => return Err(e),
    }

    let message_addr_parts: Vec<&str> = message_addr.split('/').collect();
    let method_addr_parts: Vec<&str> = method_addr.split('/').collect();

    if message_addr_parts.len() != method_addr_parts.len() {
        // Both addresses must have the same amount of parts
        return Ok(false);
    }

    for i in 1..message_addr_parts.len() {
        //println!("Parts: '{}' '{}' {}", message_addr_parts[i], method_addr_parts[i], match_part(message_addr_parts[i], method_addr_parts[i]));
        if !match_part(message_addr_parts[i], method_addr_parts[i]) {
            return Ok(false);
        }
    }

    Ok(true)
}
