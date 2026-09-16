mod discovery;
mod proxy;

use quip::server::{
    tcp::TCPServer,
    discovery::DiscoveryServer,
};

use tokio::net::TcpStream;

const DEVICE_NAME: &[u8] = b"Quippi\0"; // Qu-IP Pi

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1) // allow user to specify address
        .unwrap_or_else(|| "0.0.0.0:51325".to_string());

    // find the real Qu first
    let qu_address = discovery::find_qu().await?;
    let qu_ip = qu_address.ip();
    println!("[quippi] Real Qu: {qu_ip}");

    // connect to Qu
    let qu = TcpStream::connect(qu_address).await?;
    println!("[quippi] Connected to Qu: {qu_ip}");

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

    // spin up TCP server
    let proxy = proxy::Proxy::new(qu);
    let server = TCPServer::bind(&address).await?;
    println!("[quippi] Listening on TCP {address}");

    server
        .run(move |stream| {
            let proxy = proxy.clone();

            async move {
                proxy.accept_client(stream).await
            }
        })
        .await?;

    Ok(())
}
