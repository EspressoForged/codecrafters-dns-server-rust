use crate::protocol::DnsMessage;
use crate::error::{Error, Result};
use nom::multi::count;

mod header;
mod name;
mod question;
mod record;

// This is the main entry point for decoding a raw byte buffer.
pub fn parse_message(input: &[u8]) -> Result<DnsMessage> {
    // First, parse the fixed-size header.
    let (remaining, header) = header::parse_header(input)
        .map_err(|e| Error::ParseError(e.to_string()))?;
    
    // Using the counts from the header, parse the questions.
    // We must call `question::parse_question(input)` here to create a parser
    // that has captured the entire original input slice for handling name compression.
    let (remaining, questions) = count(question::parse_question(input), header.question_count as usize)(remaining)
        .map_err(|e| Error::ParseError(e.to_string()))?;

    // Likewise, parse the answers using the same technique.
    let (_, answers) = count(record::parse_record(input), header.answer_count as usize)(remaining)
        .map_err(|e| Error::ParseError(e.to_string()))?;

    Ok(DnsMessage {
        header,
        questions,
        answers,
    })
}