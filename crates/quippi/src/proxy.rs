use std::{
    collections::HashMap,
    io,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

type ClientID = u64;

enum Event {
    AddClient {
        reader: ReadHalf<TcpStream>,
        writer: WriteHalf<TcpStream>,
    },
    ClientData {
        id: ClientID,
        data: Vec<u8>,
    },
    ClientDisconnected {
        id: ClientID,
    },
    QuData {
        data: Vec<u8>,
    },
    QuDisconnected,
}

struct State {
    qu_writer: WriteHalf<TcpStream>,
    clients: HashMap<ClientID, WriteHalf<TcpStream>>,
}

/// Central proxy responsible for routing traffic between the real Qu
/// and all connected clients.
#[derive(Clone)]
pub struct Proxy {
    events: mpsc::Sender<Event>,
}

impl Proxy {
    pub fn new(qu: TcpStream) -> (Self, oneshot::Receiver<()>) {
        let (events, receiver) = mpsc::channel(256);
        let (stopped, stopped_receiver) = oneshot::channel();

        let proxy = Self {
            events: events.clone(),
        };

        tokio::spawn(Self::run(
            qu,
            receiver,
            events,
            stopped,
        ));

        (proxy, stopped_receiver)
    }

    /// Accept a new client connection.
    pub async fn accept_client(&self, stream: TcpStream) -> io::Result<()> {
        let (reader, writer) = tokio::io::split(stream);

        self.events
            .send(Event::AddClient { reader, writer })
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "proxy event loop has stopped",
                )
            })?;

        Ok(())
    }

    async fn run(
        qu: TcpStream,
        mut events: mpsc::Receiver<Event>,
        event_sender: mpsc::Sender<Event>,
        stopped: oneshot::Sender<()>,
    ) {
        let (qu_reader, qu_writer) = tokio::io::split(qu);

        let mut state = State {
            qu_writer,
            clients: HashMap::new(),
        };

        let mut next_client_id: ClientID = 0;

        Self::spawn_qu_reader(qu_reader, event_sender.clone());

        while let Some(event) = events.recv().await {
            match event {
                Event::AddClient { reader, writer } => {
                    // new client
                    let id = next_client_id;
                    next_client_id += 1;

                    println!("[quippi] Client {id} connected");

                    state.clients.insert(id, writer);

                    Self::spawn_client_reader(
                        id,
                        reader,
                        event_sender.clone(),
                    );
                }

                Event::ClientData { id, data } => {
                    // data from client
                    if let Err(error) = state.qu_writer.write_all(&data).await {
                        eprintln!(
                            "[quippi] Failed to write client {id} data to Qu: {error}"
                        );
                    }
                }

                Event::ClientDisconnected { id } => {
                    // terminate client
                    println!("[quippi] Client {id} disconnected");
                    state.clients.remove(&id);
                }

                Event::QuData { data } => {
                    // data from Qu
                    for (id, writer) in &mut state.clients {
                        if let Err(error) = writer.write_all(&data).await {
                            eprintln!(
                                "[quippi] Failed to write Qu data to client {id}: {error}"
                            );
                        }
                    }
                }

                Event::QuDisconnected => {
                    // terminate Qu
                    eprintln!("[quippi] Qu disconnected");
                    break;
                }
            }
        }

        eprintln!("[quippi] Proxy event loop stopped");
        let _ = stopped.send(());
    }

    fn spawn_qu_reader(
        mut reader: ReadHalf<TcpStream>,
        events: mpsc::Sender<Event>,
    ) {
        // listen to Qu
        tokio::spawn(async move {
            let mut buffer = [0u8; 8192];

            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => {
                        let _ = events.send(Event::QuDisconnected).await;
                        break;
                    }

                    Ok(length) => {
                        let data = buffer[..length].to_vec();

                        if events
                            .send(Event::QuData { data })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }

                    Err(error) => {
                        eprintln!(
                            "[quippi] Failed to read from Qu: {error}"
                        );

                        let _ = events
                            .send(Event::QuDisconnected)
                            .await;

                        break;
                    }
                }
            }
        });
    }

    fn spawn_client_reader(
        id: ClientID,
        mut reader: ReadHalf<TcpStream>,
        events: mpsc::Sender<Event>,
    ) {
        // listen to clients
        tokio::spawn(async move {
            let mut buffer = [0u8; 8192];

            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => {
                        let _ = events
                            .send(Event::ClientDisconnected { id })
                            .await;

                        break;
                    }

                    Ok(length) => {
                        let data = buffer[..length].to_vec();

                        if events
                            .send(Event::ClientData { id, data })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }

                    Err(error) => {
                        eprintln!(
                            "[quippi] Failed to read from client {id}: {error}"
                        );

                        let _ = events
                            .send(Event::ClientDisconnected { id })
                            .await;

                        break;
                    }
                }
            }
        });
    }
}
