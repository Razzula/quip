//! Squib Qu mixer emulator executable.
//!
//! Starts the Squib TCP server and dispatches incoming connections to the
//! Qu mixer emulator.

mod emulator;

use emulator::qu16;
use quip::server::{
    discovery::DiscoveryServer,
    tcp::TCPServer,
};

/// Device name reported during QuYou discovery.
const DEVICE_NAME: &[u8] = b"squib\0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:51325".to_string());

    // Start QuYou discovery.
    tokio::spawn(async {
        match DiscoveryServer::bind_default(DEVICE_NAME).await {
            Ok(server) => {
                if let Err(error) = server.run().await {
                    eprintln!("QuYou discovery server failed: {error}");
                }
            }
            Err(error) => {
                eprintln!("QuYou discovery server failed to bind: {error}");
            }
        }
    });

    // Start the Qu TCP server.
    let server = TCPServer::bind(&address).await?;

    println!("[qu-16 ] Qu Emulator listening on {address}");

    server.run(qu16::handle_client).await?;

    Ok(())
}
