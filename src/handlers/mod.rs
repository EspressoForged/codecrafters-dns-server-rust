use crate::protocol::DnsMessage;
use crate::error::Result;
use std::future::Future;
use std::pin::Pin;

// Declare the forwarder module so it can be used by other parts of the crate.
pub mod forwarder;

// The trait now uses a "GAT" (Generic Associated Type) style to return a Sendable future.
pub trait QueryHandler: Send + Sync {
    fn handle_query<'a>(
        &'a self,
        query: &'a DnsMessage,
    ) -> Pin<Box<dyn Future<Output = Result<DnsMessage>> + Send + 'a>>;
}

pub struct StagedResponseHandler;

// We implement the trait by returning a pinned, boxed, async block.
impl QueryHandler for StagedResponseHandler {
    fn handle_query<'a>(
        &'a self,
        query: &'a DnsMessage,
    ) -> Pin<Box<dyn Future<Output = Result<DnsMessage>> + Send + 'a>> {
        Box::pin(async move {
            let mut response = DnsMessage {
                header: query.header.clone(),
                questions: query.questions.clone(),
                answers: vec![],
            };
            response.header.qr = true;
            response.header.recursion_available = false;

            if query.header.opcode != 0 {
                response.header.rcode = crate::protocol::ResponseCode::NotImplemented;
                return Ok(response);
            }
            
            for q in &query.questions {
                let answer = crate::protocol::ResourceRecord {
                    name: q.name.clone(),
                    q_type: crate::protocol::QueryType::A,
                    q_class: crate::protocol::QueryClass::IN,
                    ttl: 60,
                    data: bytes::Bytes::from_static(&[8, 8, 8, 8]),
                };
                response.answers.push(answer);
            }

            Ok(response)
        })
    }
}