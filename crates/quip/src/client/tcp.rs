//! TCP client for communicating with a Qu mixer.
//!
//! Provides the client-side networking layer for connecting to a Qu mixer
//! over LAN and exchanging Qu MIDI protocol messages.

use std::io;

use qu::{
    get_system_state,
    parser::Parser,
    QuEvent,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug)]
pub enum QuipError {
    Io(io::Error),
}

impl From<io::Error> for QuipError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct Quip {
    stream: TcpStream,
    parser: Parser,
}

impl Quip {
    pub async fn connect(address: impl AsRef<str>) -> Result<Self, QuipError> {
        let stream = TcpStream::connect(address.as_ref()).await?;

        Ok(Self {
            stream,
            parser: Parser::new(),
        })
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), QuipError> {
        self.stream.write_all(data).await?;
        Ok(())
    }

    pub async fn request_system_state(&mut self) -> Result<(), QuipError> {
        self.send(&get_system_state()).await
    }

    pub async fn next_events(&mut self) -> Result<Vec<QuEvent>, QuipError> {
        let mut buffer = [0u8; 4096];

        let count = self.stream.read(&mut buffer).await?;

        if count == 0 {
            return Ok(Vec::new());
        }

        Ok(self.parser.push(&buffer[..count]))
    }

    pub async fn run<F>(&mut self, mut callback: F) -> Result<(), QuipError>
    where
        F: FnMut(QuEvent),
    {
        let mut buffer = [0u8; 4096];

        loop {
            let count = self.stream.read(&mut buffer).await?;

            if count == 0 {
                return Ok(());
            }

            for event in self.parser.push(&buffer[..count]) {
                callback(event);
            }
        }
    }

    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}
