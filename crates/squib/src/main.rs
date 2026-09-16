//! Squib Qu mixer emulator executable.
//!
//! Starts the Squib TCP server and dispatches incoming connections to the
//! Qu mixer emulator.

mod emulator;

use emulator::{qu16, qufind};
use quip::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:51325".to_string());

    // Start QuYou discovery.
    tokio::spawn(async {
        if let Err(error) = qufind::run().await {
            eprintln!("QuYou discovery server failed: {error}");
        }
    });

    // Start the Qu TCP server.
    let server = Server::bind(&address).await?;

    println!("[qu-16 ] Qu Emulator listening on {address}");

    server.run(qu16::handle_client).await?;

    Ok(())
}
