mod discovery;
mod proxy;

use quip::server::{
    discovery::DiscoveryServer,
    tcp::TCPServer,
};

use std::time::Duration;
use tokio::net::TcpStream;

const DEVICE_NAME: &[u8] = b"Quippi\0"; // Qu-IP Pi

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1) // allow user to specify address
        .unwrap_or_else(|| "0.0.0.0:51325".to_string());

    // spin up QuYou discovery server
    tokio::spawn(async {
        match DiscoveryServer::bind_default(DEVICE_NAME).await {
            Ok(server) => {
                if let Err(error) = server.run().await {
                    eprintln!("[qufind] Discovery server failed: {error}");
                }
            }
            Err(error) => {
                eprintln!("[qufind] Discovery server failed to bind: {error}");
            }
        }
    });

    loop {
        // find the real Qu
        let qu = loop {
            match discovery::find_qu().await {
                Ok(qu_address) => {
                    let qu_ip = qu_address.ip();
                    println!("[quippi] Real Qu: {qu_ip}");

                    // connect to Qu
                    match TcpStream::connect(qu_address).await {
                        Ok(qu) => {
                            println!("[quippi] Connected to Qu: {qu_ip}");
                            break qu;
                        }

                        Err(error) => {
                            eprintln!(
                                "[quippi] Failed to connect to Qu at {qu_ip}: {error}"
                            );
                        }
                    }
                }

                Err(error) => {
                    eprintln!("[quippi] Failed to find Qu: {error}");
                }
            }

            println!("[quippi] Retrying in 15 seconds...");
            tokio::time::sleep(Duration::from_secs(15)).await;
        };

        // spin up TCP server
        let (proxy, proxy_stopped) = proxy::Proxy::new(qu);
        let server = TCPServer::bind(&address).await?;
        println!("[quippi] Listening on TCP {address}");

        tokio::select! {
            result = server.run(move |stream| {
                let proxy = proxy.clone();

                async move {
                    proxy.accept_client(stream).await
                }
            }) => {
                result?;
                println!("[quippi] TCP server stopped");
            }

            _ = proxy_stopped => {
                println!("[quippi] Proxy stopped; restarting...");
            }
        }
    }
}
