//! QuYou discovery client.
//!
//! Discovers Qu mixers using the QuYou UDP discovery protocol.

use qu::protocol::{DISCOVERY_MESSAGE, DISCOVERY_PORT};

use std::{
    io,
    net::SocketAddr,
    time::Duration,
};

use tokio::{
    net::UdpSocket,
    time::timeout,
};

/// UDP port used as the client-side source port for discovery.
const DISCOVERY_CLIENT_PORT: u16 = 51321;
/// How long to wait for discovery responses.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

/// A Qu mixer discovered on the network.
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub address: SocketAddr,
}

/// Discover Qu mixers on the local network.
///
/// Sends a `QU Find` broadcast and waits for responses for three seconds.
/// Multiple responses are collected and returned.
pub async fn discover() -> io::Result<Vec<Device>> {
    let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_CLIENT_PORT)).await?;

    socket.set_broadcast(true)?;

    let destination = format!("255.255.255.255:{DISCOVERY_PORT}");

    socket
        .send_to(DISCOVERY_MESSAGE, &destination)
        .await?;

    println!(
        "[qufind] TX {} -> {}: {}",
        socket.local_addr()?,
        destination,
        String::from_utf8_lossy(DISCOVERY_MESSAGE),
    );

    let mut devices = Vec::new();
    let mut buffer = [0u8; 4096];

    loop {
        let result = timeout(
            DISCOVERY_TIMEOUT,
            socket.recv_from(&mut buffer),
        )
        .await;

        let (count, address) = match result {
            Ok(result) => result?,
            Err(_) => break,
        };

        let data = &buffer[..count];

        let name = data
            .strip_suffix(&[0])
            .unwrap_or(data);

        let name = String::from_utf8_lossy(name).into_owned();

        println!(
            "[qufind] RX {} -> {}: {}",
            address,
            socket.local_addr()?,
            name,
        );

        devices.push(Device {
            name,
            address,
        });
    }

    Ok(devices)
}
