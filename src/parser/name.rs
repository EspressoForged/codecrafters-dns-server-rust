use nom::{
    bytes::complete::take,
    number::complete::u8,
    IResult,
};

/// Recursively parses a domain name.
///
/// Each call to this function parses one component:
/// - A pointer (starts with 0b11...): Jumps to another location and parses from there.
/// - A label: Reads a length byte `L` and `L` bytes of content, then recursively calls itself for the rest.
/// - A null byte: Terminates the recursion, returning an empty string.
fn parse_name_recursive<'a>(
    original_input: &'a [u8], // The complete DNS packet buffer for pointer jumps
    input: &'a [u8],          // The current position we are parsing from
    depth: u8,                // Recursion depth to prevent infinite loops
) -> IResult<&'a [u8], String> {
    // Prevent infinite loops from malformed pointer cycles
    if depth > 10 {
        return Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TooLarge,
        )));
    }

    // Peek at the first byte to decide what to do
    let (input_after_peek, first_byte) = u8(input)?;

    // Case 1: The name is a pointer to another location
    if (first_byte & 0xC0) == 0xC0 {
        // Pointers are 2 bytes. We already read the first, now read the second.
        let (input_after_ptr, second_byte) = u8(input_after_peek)?;
        let offset = (((first_byte & 0x3F) as u16) << 8 | second_byte as u16) as usize;

        // Recursively parse from the new offset using the *original* buffer.
        // The main `input` stream continues from *after* the 2-byte pointer.
        let (_, pointed_name) =
            parse_name_recursive(original_input, &original_input[offset..], depth + 1)?;
        return Ok((input_after_ptr, pointed_name));
    }

    // Case 2: This is the end of the name (the null terminator)
    if first_byte == 0 {
        return Ok((input_after_peek, String::new()));
    }

    // Case 3: It's a standard label.
    // The first byte is the length of the label.
    let (input_after_label, label_data) = take(first_byte)(input_after_peek)?;
    let label_str = String::from_utf8_lossy(label_data).to_string();

    // After parsing this label, recursively parse the rest of the name.
    let (final_input, rest_of_name) =
        parse_name_recursive(original_input, input_after_label, depth)?;

    // Combine the current label with the rest of the name.
    if rest_of_name.is_empty() {
        Ok((final_input, label_str))
    } else {
        Ok((final_input, format!("{}.{}", label_str, rest_of_name)))
    }
}

/// Creates a nom parser for a domain name.
/// It captures the original input buffer to handle pointers correctly.
pub fn parse_name<'a>(original_input: &'a [u8]) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], String> {
    move |input: &'a [u8]| parse_name_recursive(original_input, input, 0)
}