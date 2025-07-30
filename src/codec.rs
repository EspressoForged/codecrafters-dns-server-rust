use crate::protocol::*;
use bytes::{BufMut, Bytes, BytesMut};

pub fn encode_message(message: &DnsMessage) -> Bytes {
    let mut buffer = BytesMut::new();
    encode_header(&mut buffer, &message.header, message.questions.len(), message.answers.len());
    for question in &message.questions {
        encode_question(&mut buffer, question);
    }
    for answer in &message.answers {
        encode_record(&mut buffer, answer);
    }
    buffer.freeze()
}

fn encode_header(buffer: &mut BytesMut, header: &DnsHeader, qd_count: usize, an_count: usize) {
    buffer.put_u16(header.id);
    let mut flags: u16 = 0;
    if header.qr { flags |= 1 << 15; }
    flags |= (header.opcode as u16) << 11;
    if header.authoritative_answer { flags |= 1 << 10; }
    if header.truncation { flags |= 1 << 9; }
    if header.recursion_desired { flags |= 1 << 8; }
    if header.recursion_available { flags |= 1 << 7; }
    flags |= header.rcode as u16;
    buffer.put_u16(flags);
    buffer.put_u16(qd_count as u16);
    buffer.put_u16(an_count as u16);
    buffer.put_u16(header.authority_count);
    buffer.put_u16(header.additional_count);
}

fn encode_name(buffer: &mut BytesMut, name: &str) {
    for label in name.split('.') {
        buffer.put_u8(label.len() as u8);
        buffer.put_slice(label.as_bytes());
    }
    buffer.put_u8(0); // Null terminator for the name
}

fn encode_question(buffer: &mut BytesMut, question: &DnsQuestion) {
    encode_name(buffer, &question.name);
    buffer.put_u16(question.q_type as u16);
    buffer.put_u16(question.q_class as u16);
}

fn encode_record(buffer: &mut BytesMut, record: &ResourceRecord) {
    encode_name(buffer, &record.name);
    buffer.put_u16(record.q_type as u16);
    buffer.put_u16(record.q_class as u16);
    buffer.put_u32(record.ttl);
    buffer.put_u16(record.data.len() as u16);
    buffer.put_slice(&record.data);
}