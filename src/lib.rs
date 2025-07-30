// Re-export key components to make them accessible from the binary crate
pub use error::{Error, Result};
// pub use handlers::QueryHandler;
pub use server::DnsServer;

// Declare the modules that make up our library
pub mod codec;
pub mod error;
pub mod handlers;
pub mod parser;
pub mod protocol;
pub mod server;