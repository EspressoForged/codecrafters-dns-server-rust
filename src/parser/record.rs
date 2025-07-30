use crate::protocol::{QueryClass, QueryType, ResourceRecord};
use super::name::parse_name;
use nom::{bytes::complete::take, number::complete::{be_u16, be_u32}, sequence::tuple, IResult};
use std::convert::TryFrom;
use bytes::Bytes;

pub fn parse_record<'a>(original_input: &'a [u8]) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ResourceRecord> {
    move |input: &'a [u8]| {
        // First, parse the components that come before the variable-length data.
        let (input, (name, q_type_val, q_class_val, ttl, rd_len)) = tuple((
            parse_name(original_input),
            be_u16,
            be_u16,
            be_u32,
            be_u16,
        ))(input)?;

        // Now, use the parsed rd_len to parse the rdata field.
        let (input, rdata) = take(rd_len)(input)?;

        // Construct the final ResourceRecord.
        let record = ResourceRecord {
            name,
            q_type: QueryType::try_from(q_type_val).unwrap_or(QueryType::A),
            q_class: QueryClass::try_from(q_class_val).unwrap_or(QueryClass::IN),
            ttl,
            data: Bytes::copy_from_slice(rdata),
        };

        Ok((input, record))
    }
}