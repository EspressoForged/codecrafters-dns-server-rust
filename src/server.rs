use super::error::Result;
use super::handlers::QueryHandler;
use super::codec::encode_message;
use super::parser::parse_message;
use std::sync::Arc;
use tokio::net::UdpSocket;
use log::{info, error};

// The struct is now generic over any type that implements QueryHandler.
pub struct DnsServer<H: QueryHandler> {
    socket: Arc<UdpSocket>,
    handler: Arc<H>,
}

impl<H: QueryHandler + 'static> DnsServer<H> {
    pub async fn new(handler: H) -> Result<Self> {
        let socket = UdpSocket::bind("127.0.0.1:2053").await?;
        info!("DNS server listening on 127.0.0.1:2053");
        Ok(Self { socket: Arc::new(socket), handler: Arc::new(handler) })
    }

    pub async fn run(self) -> Result<()> {
        let mut buf = [0; 512];
        loop {
            let (size, source) = self.socket.recv_from(&mut buf).await?;
            info!("Received {} bytes from {}", size, source);

            let handler = self.handler.clone();
            let socket = self.socket.clone();
            let packet = buf[..size].to_vec();

            tokio::spawn(async move {
                match parse_message(&packet) {
                    Ok(query) => {
                        match handler.handle_query(&query).await {
                            Ok(response) => {
                                let response_bytes = encode_message(&response);
                                if let Err(e) = socket.send_to(&response_bytes, source).await {
                                    error!("Failed to send response: {}", e);
                                }
                            }
                            Err(e) => error!("Handler failed: {}", e),
                        }
                    }
                    Err(e) => error!("Failed to parse message: {}", e),
                }
            });
        }
    }
}