use nom::{
    branch::alt,
    combinator::recognize,
    multi::many_till, // `count` is in `multi`
    number::complete::{be_u16, u8},
    sequence::preceded,
    bytes::complete::take, // `take` is for byte slices
    IResult,
};

// This helper now correctly takes a slice and returns a slice
fn parse_label(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, len) = u8(input)?;
    take(len)(input)
}

fn parse_name_recursive<'a>(
    original_input: &'a [u8],
    input: &'a [u8],
    depth: u8,
) -> IResult<&'a [u8], String> {
    if depth > 10 { // recursion limit
        return Err(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)));
    }

    // `many_till` will read labels until it finds a null byte (the second parser).
    let (input, (labels, _)) = many_till(
        // We recognize either a normal label or a pointer start byte.
        recognize(alt((
            parse_label,
            // A pointer is just 2 bytes starting with 0b11...
            recognize(preceded(take(1u8), take(1u8))),
        ))),
        // The terminator is a single null byte.
        u8,
    )(input)?;

    let mut name = String::new();
    let mut needs_dot = false;
    for label_slice in labels {
        if (label_slice[0] & 0xC0) == 0xC0 { // It's a pointer
            // The slice is 2 bytes long, parse it as a u16.
            let (_, offset) = be_u16(label_slice)?;
            let offset = (offset & 0x3FFF) as usize;

            // Recursively parse from the new offset.
            let (_, pointed_name) = parse_name_recursive(original_input, &original_input[offset..], depth + 1)?;
            if needs_dot { name.push('.'); }
            name.push_str(&pointed_name);
            // After a pointer, the name is complete.
            return Ok((input, name));
        } else { // It's a standard label
            let len = label_slice[0] as usize;
            if needs_dot { name.push('.'); }
            // Slice the content from after the length byte.
            name.push_str(&String::from_utf8_lossy(&label_slice[1..=len]));
            needs_dot = true;
        }
    }
    Ok((input, name))
}

pub fn parse_name<'a>(original_input: &'a[u8]) -> impl Fn(&'a[u8]) -> IResult<&'a [u8], String> {
    move |input: &'a [u8]| parse_name_recursive(original_input, input, 0)
}