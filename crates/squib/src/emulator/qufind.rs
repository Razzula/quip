//! QuYou discovery protocol emulator.
//!
//! Listens for QuYou discovery broadcasts on UDP port 51320 and responds
//! with the virtual mixer's device name.

use qu::{
    protocol::{DISCOVERY_PORT, DISCOVERY_MESSAGE},
    messages::hex,
};

use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io,
    net::SocketAddr,
};
use tokio::net::UdpSocket;

/// Device name reported during QuYou discovery.
const DEVICE_NAME: &[u8] = b"squib\0";

/// Run the QuYou discovery responder.
///
/// QuYou broadcasts `QU Find` to UDP port 51320. A real Qu responds
/// directly to the source IP and source port of the request with its
/// NUL-terminated device name.
pub async fn run() -> io::Result<()> {
    let socket = Socket::new(
        Domain::IPV4,
        Type::DGRAM,
        Some(Protocol::UDP),
    )?;
    socket.set_nonblocking(true)?;
    socket.set_reuse_address(true)?;
    socket.set_ttl_v4(255)?;
    let address = SocketAddr::from(([0, 0, 0, 0], DISCOVERY_PORT));
    socket.bind(&address.into())?;
    let socket = UdpSocket::from_std(socket.into())?;

    println!(
        "[qufind] Discovery emulator listening on UDP {}",
        DISCOVERY_PORT
    );

    let mut buffer = [0u8; 4096];

    loop {
        let (count, addr) = socket.recv_from(&mut buffer).await?;

        let data = &buffer[..count];

        println!(
            "[qufind] RX from {}: {}",
            addr, hex(data),
        );

        if data != DISCOVERY_MESSAGE {
            continue;
        }

        println!("[qufind] Discovery request received from {addr}");

        // The real Qu responds to the source IP and source UDP port
        // of the discovery request, rather than broadcasting the response.
        socket.send_to(DEVICE_NAME, addr).await?;

        println!(
            "[qufind] TX to {}: {}",
            addr, hex(DEVICE_NAME),
        );
    }
}
