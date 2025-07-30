use clap::Parser;
use codecrafters_dns_server::handlers::{StagedResponseHandler};
use codecrafters_dns_server::handlers::forwarder::ForwardingHandler;
use codecrafters_dns_server::{Result, DnsServer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    resolver: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!("Logs from your program will appear here!");

    let cli = Cli::parse();
    
    // The logic is now cleaner, using a concrete type for each branch.
    if let Some(resolver_str) = cli.resolver {
        let handler = ForwardingHandler::new(resolver_str).await?;
        let server = DnsServer::new(handler).await?;
        server.run().await?;
    } else {
        let handler = StagedResponseHandler{};
        let server = DnsServer::new(handler).await?;
        server.run().await?;
    };
    
    Ok(())
}