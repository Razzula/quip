//! Squib Qu mixer emulator executable.
//!
//! Starts the Squib TCP server and dispatches incoming connections to the
//! Qu mixer emulator.

mod emulator;

use emulator::handle_client;
use quip::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:51325".to_string());

    let server = Server::bind(&address).await?;

    println!("Qu emulator listening on {address}");

    server.run(handle_client).await?;

    Ok(())
}
