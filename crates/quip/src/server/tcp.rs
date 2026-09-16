//! Generic TCP server infrastructure for Qu communication.
//!
//! Provides reusable server-side networking for accepting TCP connections
//! and dispatching each connection to a caller-provided handler.

use std::{future::Future, io, sync::Arc};

use tokio::net::{TcpListener, TcpStream};

pub struct TCPServer {
    listener: Arc<TcpListener>,
}

impl TCPServer {
    pub async fn bind(address: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(address).await?;

        Ok(Self {
            listener: Arc::new(listener),
        })
    }

    pub async fn run<F, Fut>(&self, handler: F) -> io::Result<()>
    where
        F: Fn(TcpStream) -> Fut + Send + Sync + Copy + 'static,
        Fut: Future<Output = io::Result<()>> + Send + 'static,
    {
        loop {
            let (stream, address) = self.listener.accept().await?;

            println!("[qu    ] Client connected: {address}");

            tokio::spawn(async move {
                if let Err(error) = handler(stream).await {
                    eprintln!("[qu    ] Client error: {error}");
                }

                println!("[qu    ] Client disconnected: {address}");
            });
        }
    }
}
