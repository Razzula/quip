//! Qu mixer UDP discovery server.
//!
//! Listens for QuYou discovery broadcasts on UDP port 51320 and responds
//! with the configured device name.

use qu::{
    messages::hex,
    protocol::{DISCOVERY_MESSAGE, DISCOVERY_PORT},
};

use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io,
    net::SocketAddr,
};

use tokio::net::UdpSocket;

/// UDP server for QuYou mixer discovery.
pub struct DiscoveryServer {
    socket: UdpSocket,
    device_name: Vec<u8>,
}

impl DiscoveryServer {
    /// Bind a discovery server to the specified address.
    ///
    /// `device_name` is the NUL-terminated device name returned to QuYou.
    pub async fn bind(address: &str, device_name: &[u8]) -> io::Result<Self> {
        let socket = Socket::new(
            Domain::IPV4,
            Type::DGRAM,
            Some(Protocol::UDP),
        )?;

        socket.set_nonblocking(true)?;
        socket.set_reuse_address(true)?;
        socket.set_ttl_v4(255)?;

        let address: SocketAddr = address.parse().map_err(|error| {
            io::Error::new(io::ErrorKind::InvalidInput, error)
        })?;

        socket.bind(&address.into())?;

        let socket = UdpSocket::from_std(socket.into())?;

        Ok(Self {
            socket,
            device_name: device_name.to_vec(),
        })
    }

    /// Bind a discovery server to the default Qu discovery port.
    pub async fn bind_default(device_name: &[u8]) -> io::Result<Self> {
        Self::bind(
            &format!("0.0.0.0:{DISCOVERY_PORT}"),
            device_name,
        )
        .await
    }

    /// Run the discovery server.
    pub async fn run(&self) -> io::Result<()> {
        let mut buffer = [0u8; 4096];

        println!(
            "[qufind] Listening on UDP {}",
            self.socket.local_addr()?
        );

        loop {
            let (count, address) = self.socket.recv_from(&mut buffer).await?;

            let data = &buffer[..count];

            println!(
                "[qufind] RX from {}: {}",
                address,
                hex(data),
            );

            if data != DISCOVERY_MESSAGE {
                continue;
            }

            println!(
                "[qufind] Discovery request received from {address}"
            );

            // A real Qu responds directly to the source IP and source
            // UDP port of the discovery request.
            self.socket
                .send_to(&self.device_name, address)
                .await?;

            println!(
                "[qufind] TX to {}: {}",
                address,
                hex(&self.device_name),
            );
        }
    }
}
