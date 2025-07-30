use super::QueryHandler;
use crate::codec::encode_message;
use crate::parser::parse_message;
use crate::protocol::{DnsMessage, ResponseCode};
use crate::error::{Result, Error};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::net::UdpSocket;

pub struct ForwardingHandler {
    resolver_addr: SocketAddr,
}

impl ForwardingHandler {
    pub async fn new(resolver_str: String) -> Result<Self> {
        let resolver_addr = tokio::net::lookup_host(&resolver_str).await?
            .next()
            .ok_or_else(|| Error::InvalidResolverAddress(resolver_str.clone()))?;
        Ok(Self { resolver_addr })
    }
}

impl QueryHandler for ForwardingHandler {
    fn handle_query<'a>(
        &'a self,
        query: &'a DnsMessage,
    ) -> Pin<Box<dyn Future<Output = Result<DnsMessage>> + Send + 'a>> {
        Box::pin(async move {
            if query.header.opcode != 0 {
                let mut response = query.clone();
                response.header.qr = true;
                response.header.rcode = ResponseCode::NotImplemented;
                return Ok(response);
            }

            let mut final_response = query.clone();
            final_response.header.qr = true;
            final_response.header.recursion_available = true;
            final_response.answers.clear();

            for question in &query.questions {
                let mut single_query = query.clone();
                single_query.questions = vec![question.clone()];
                
                let query_bytes = encode_message(&single_query);

                let socket = UdpSocket::bind("0.0.0.0:0").await?;
                socket.send_to(&query_bytes, self.resolver_addr).await?;

                let mut response_buf = [0u8; 512];
                let (len, _) = socket.recv_from(&mut response_buf).await?;

                let resolver_response = parse_message(&response_buf[..len])?;
                final_response.answers.extend(resolver_response.answers);
            }
            Ok(final_response)
        })
    }
}