use crate::protocol::{DnsQuestion, QueryClass, QueryType};
use super::name::parse_name;
use nom::{combinator::map, number::complete::be_u16, sequence::tuple, IResult};
use std::convert::TryFrom;

pub fn parse_question<'a>(original_input: &'a [u8]) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], DnsQuestion> {
    map(
        tuple((parse_name(original_input), be_u16, be_u16)),
        |(name, q_type_val, q_class_val)| DnsQuestion {
            name,
            q_type: QueryType::try_from(q_type_val).unwrap_or(QueryType::A),
            q_class: QueryClass::try_from(q_class_val).unwrap_or(QueryClass::IN),
        },
    )
}