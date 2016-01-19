/// Returns the 4-byte padded position
///
/// # Example
///
/// ```
/// use rosc::utils;
/// let pos: u64 = 10;
/// assert_eq!(12u64, utils::pad(pos))
/// ```
pub fn pad(pos: u64) -> u64 {
    match pos % 4 {
        0 => pos,
        d => pos + (4 - d),
    }
}
