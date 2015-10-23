use {osc_types, errors};

/// Common MTP size for ethernet
pub const MTP: usize = 1536;

pub fn destruct(message: &mut [u8], size: usize) -> Result<osc_types::OscPacket, errors::OscError> {
    Ok(osc_types::OscPacket::OscMessage)
}