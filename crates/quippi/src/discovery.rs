use std::io;
use std::net::SocketAddr;

use qu::protocol::{TCP_PORT};
use quip::client::discovery::discover;

pub async fn find_qu() -> io::Result<SocketAddr> {
    println!("[qufind] Looking for Qu...");

    // search for Qus
    let devices = discover().await?;
    let device = devices
        .into_iter()
        .min_by_key(|device| {
            match device.name.to_lowercase().as_str() {
                // resolve multiple devices in a hierarchy
                "squib" => 1,
                "quippi" => 2, // chances of wanted to connect quippi to another quippi is low
                _ => 0, // prioiritse real devices
            }
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "No Qu mixers found",
            )
        })?;

    // identify selected device
    let address = SocketAddr::new(
        device.address.ip(),
        TCP_PORT,
    );
    println!("[qufind] Found {} at {}", device.name, address);

    Ok(address)
}
