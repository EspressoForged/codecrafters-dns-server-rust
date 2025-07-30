use crate::protocol::{DnsHeader, ResponseCode};
use nom::{
    bits::bits,
    bits::complete::{bool, take},
    bytes::complete::take as take_bytes,
    error::Error as NomError,
    number::complete::be_u16,
    sequence::tuple,
    IResult,
};
use std::convert::TryFrom;

// A type alias for the complex tuple that our flags parser returns.
// This improves readability significantly.
type DnsFlags = (bool, u8, bool, bool, bool, bool, u8);

fn parse_flags(input: &[u8]) -> IResult<&[u8], DnsFlags> {
    let (input, flags_slice) = take_bytes(2usize)(input)?;
    
    // We now provide a full type annotation for the tuple's output.
    // This tells `nom` exactly what integer types to infer for each `take`.
    let (_, (qr, opcode, aa, tc, rd, ra, _z, rcode_val)): (_, (bool, u8, bool, bool, bool, bool, u8, u8)) =
        bits::<_, _, NomError<(&[u8], usize)>, _, _>(tuple((
            bool,
            take(4u8), // infers u8
            bool,
            bool,
            bool,
            bool,
            take(3u8), // infers u8
            take(4u8), // infers u8
        )))(flags_slice)?;

    Ok((input, (qr, opcode, aa, tc, rd, ra, rcode_val)))
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], DnsHeader> {
    let (input, (id, flags, qd, an, ns, ar)) = tuple((
        be_u16,
        parse_flags, // The return type is now the clean `DnsFlags` alias.
        be_u16,
        be_u16,
        be_u16,
        be_u16,
    ))(input)?;

    let (qr, opcode, aa, tc, rd, ra, rcode_val) = flags;
    let rcode = ResponseCode::try_from(rcode_val).unwrap_or(ResponseCode::FormatError);

    Ok((
        input,
        DnsHeader {
            id, qr, opcode, rcode,
            authoritative_answer: aa,
            truncation: tc,
            recursion_desired: rd,
            recursion_available: ra,
            question_count: qd,
            answer_count: an,
            authority_count: ns,
            additional_count: ar,
        },
    ))
}