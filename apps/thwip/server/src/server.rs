use std::{
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{
        broadcast,
        mpsc,
        watch,
        RwLock,
    },
};

use qu::{messages::QuEvent, protocol::TCP_PORT};

use quip::{
    client::discovery::discover,
    qu_from_state,
    qu_to_state,
    state::{ChannelRef, MixerState},
};

#[derive(Clone)]
struct AppState {
    mixer: Arc<RwLock<MixerState>>,

    /// Changes originating from the Qu which should be sent to clients.
    client_updates: broadcast::Sender<MixerChange>,

    /// Commands sent to the Qu manager.
    qu_commands: mpsc::Sender<QuCommand>,

    /// Current Qu connection state.
    qu_status: watch::Sender<QuStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MixerChange {
    Fader {
        channel: ChannelRef,
        value: Option<f32>,
    },

    Mute {
        channel: ChannelRef,
        muted: bool,
    },

    Name {
        channel: ChannelRef,
        name: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum QuStatus {
    Disconnected,

    Discovering,

    Connecting {
        name: String,
        address: String,
    },

    Synchronising,

    Connected {
        name: String,
        address: String,
    },

    Error {
        message: String,
    },
}

#[derive(Debug)]
enum QuCommand {
    ClientConnected,
    ClientDisconnected,
    Change(MixerChange),
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    QuStatus {
        #[serde(flatten)]
        status: QuStatus,
    },

    State(MixerState),

    Change(MixerChange),
}

struct ClientHandler {
    state: AppState,
}

impl ClientHandler {
    fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn handle(&self, text: &str) {
        println!("[RX UI] {}", text);

        let Ok(change) = serde_json::from_str::<MixerChange>(text) else {
            eprintln!("[RX UI] Invalid mixer change");
            return;
        };

        println!("[RX UI] Parsed {:?}", change);

        if self
            .state
            .qu_commands
            .send(QuCommand::Change(change.clone()))
            .await
            .is_err()
        {
            eprintln!("[TX QU] Qu manager is unavailable");
        } else {
            println!("[TX QU] {:?}", change);
        }
    }
}

struct QuHandler {
    state: AppState,
}

impl QuHandler {
    fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn handle(&self, event: QuEvent) -> bool {
        if !matches!(event, QuEvent::ActiveSense) {
            println!("[RX QU] {:?}", event);
        }

        let change = match &event {
            QuEvent::Fader { channel, value } => {
                let Some(channel) = ChannelRef::from_channel(*channel) else {
                    eprintln!(
                        "[RX QU] Unsupported fader channel: {:?}",
                        channel
                    );
                    return false;
                };

                let db = qu::faders::fader_to_db(*value);

                Some(MixerChange::Fader {
                    channel,
                    value: if db == f32::NEG_INFINITY {
                        None
                    } else {
                        Some(db)
                    },
                })
            }

            QuEvent::Mute { channel, muted } => {
                let Some(channel) = ChannelRef::from_channel(*channel) else {
                    eprintln!(
                        "[RX QU] Unsupported mute channel: {:?}",
                        channel
                    );
                    return false;
                };

                Some(MixerChange::Mute {
                    channel,
                    muted: *muted,
                })
            }

            QuEvent::Name { channel, name } => {
                let Some(channel) = ChannelRef::from_channel(*channel) else {
                    eprintln!(
                        "[RX QU] Unsupported name channel: {:?}",
                        channel
                    );
                    return false;
                };

                Some(MixerChange::Name {
                    channel,
                    name: name.clone(),
                })
            }

            QuEvent::ActiveSense => None,

            _ => None,
        };

        /*
         * Let quip own the authoritative interpretation of every Qu event.
         */
        {
            let mut mixer = self.state.mixer.write().await;
            qu_to_state::handle_event(&mut mixer, event.clone());
        }

        /*
         * The end-of-system-state SysEx marks completion of the initial
         * synchronisation.
         */
        let is_end_sync = matches!(
            &event,
            QuEvent::SysEx(data)
                if data.as_slice() == qu::protocol::end_sync()
        );

        if let QuEvent::SysEx(data) = &event {
            println!("[QU] SysEx received: {:02X?}", data);
            println!(
                "[QU] SysEx expected: {:02X?}",
                qu::protocol::end_sync()
            );
        }

        if is_end_sync {
            println!("[QU] Initial system state received");

            /*
             * The manager owns the status transition. Returning true tells
             * it that synchronisation has completed.
             */
            return true;
        }

        /*
         * Do not broadcast individual changes while the initial state is
         * still being populated. The complete MixerState is sent once
         * synchronisation finishes.
         */
        if let Some(change) = change {
            if matches!(
                &*self.state.qu_status.borrow(),
                QuStatus::Connected { .. }
            ) {
                println!("[QU TX → CLIENT] {:?}", change);

                if self.state.client_updates.send(change).is_err() {
                    println!("[QU TX → CLIENT] No clients connected");
                }
            }
        }

        false
    }
}

async fn send_change_to_qu(
    stream: &mut TcpStream,
    state: &AppState,
    change: &MixerChange,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel = match change {
        MixerChange::Fader { channel, .. }
        | MixerChange::Mute { channel, .. } => {
            let Some(channel) = channel.channel() else {
                eprintln!("[QU TX] Invalid channel: {:?}", channel);
                return Ok(());
            };

            channel
        }

        MixerChange::Name { .. } => {
            eprintln!("[QU TX] Name changes are not yet supported");
            return Ok(());
        }
    };

    let message = {
        let mixer = state.mixer.read().await;
        let mut mixer = mixer.clone();

        match change {
            MixerChange::Fader { value, .. } => {
                let db = value.unwrap_or(f32::NEG_INFINITY);

                mixer.set_fader(channel, db);
                qu_from_state::fader(&mixer, channel)
            }

            MixerChange::Mute { muted, .. } => {
                mixer.set_muted(channel, *muted);
                qu_from_state::mute(&mixer, channel)
            }

            MixerChange::Name { .. } => {
                return Ok(());
            }
        }
    };

    let Some(message) = message else {
        eprintln!(
            "[QU TX] Could not generate command for {:?}",
            change
        );
        return Ok(());
    };

    stream.write_all(&message).await?;

    println!("[QU TX] {:?} → {:?}", change, message);

    Ok(())
}

async fn request_system_state(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stream
        .write_all(&qu::protocol::get_system_state())
        .await?;

    println!("[QU TX] GET_SYSTEM_STATE");

    Ok(())
}

async fn discover_and_connect(
    state: &AppState,
) -> Result<(TcpStream, String), Box<dyn std::error::Error + Send + Sync>> {
    println!("[QU] Discovering mixers...");

    let devices = discover().await?;

    let device = devices
        .into_iter()
        .min_by_key(|device| {
            match device.name.to_lowercase().as_str() {
                "quippi" => 0,
                "squib" => 1,
                _ => 2,
            }
        })
        .ok_or("No Qu mixers found")?;

    /*
     * Discovery uses UDP port 51320, whereas Qu mixer control uses
     * TCP port 51325. Keep the discovered IP but use the TCP port.
     */
    let address = SocketAddr::new(
        device.address.ip(),
        TCP_PORT,
    );

    println!("[QU] Connecting to {}", address);

    let _ = state.qu_status.send(QuStatus::Connecting {
        name: device.name.clone(),
        address: address.to_string(),
    });

    let stream = TcpStream::connect(address).await?;

    println!("[QU] Connected to {}", address);

    Ok((stream, device.name))
}

async fn run_qu_manager(
    state: AppState,
    mut commands: mpsc::Receiver<QuCommand>,
) {
    let mut client_count = 0usize;
    let mut stream: Option<TcpStream> = None;
    let mut parser = qu::parser::Parser::new();
    let mut buffer = [0u8; 4096];

    let handler = QuHandler::new(state.clone());

    loop {
        /*
         * There are no clients, so there is no reason to maintain a Qu
         * connection. Wait for the next client before doing discovery.
         */
        if client_count == 0 && stream.is_none() {
            match commands.recv().await {
                Some(QuCommand::ClientConnected) => {
                    client_count += 1;

                    println!(
                        "[CLIENT] Connected; {} client(s)",
                        client_count
                    );

                    if let Err(error) =
                        connect_and_synchronise(
                            &state,
                            &mut stream,
                            &mut parser,
                            &mut buffer,
                            &handler,
                        )
                        .await
                    {
                        eprintln!(
                            "[QU] Failed to connect: {}",
                            error
                        );

                        let _ = state.qu_status.send(
                            QuStatus::Error {
                                message: error.to_string(),
                            },
                        );
                    }
                }

                Some(QuCommand::ClientDisconnected) => {
                    /*
                     * Defensive: a disconnect without a corresponding
                     * connection should simply be ignored.
                     */
                }

                Some(QuCommand::Change(change)) => {
                    eprintln!(
                        "[QU] Ignoring change while no clients are connected: {:?}",
                        change
                    );
                }

                None => break,
            }

            continue;
        }

        /*
         * A Qu connection does not exist but one or more clients do.
         * This can happen after a connection failure. A new client arriving
         * should trigger a fresh discovery attempt.
         */
        if stream.is_none() {
            match commands.recv().await {
                Some(QuCommand::ClientConnected) => {
                    client_count += 1;

                    println!(
                        "[CLIENT] Connected; {} client(s)",
                        client_count
                    );

                    if let Err(error) =
                        connect_and_synchronise(
                            &state,
                            &mut stream,
                            &mut parser,
                            &mut buffer,
                            &handler,
                        )
                        .await
                    {
                        eprintln!(
                            "[QU] Failed to connect: {}",
                            error
                        );

                        let _ = state.qu_status.send(
                            QuStatus::Error {
                                message: error.to_string(),
                            },
                        );
                    }
                }

                Some(QuCommand::ClientDisconnected) => {
                    client_count = client_count.saturating_sub(1);

                    println!(
                        "[CLIENT] Disconnected; {} client(s)",
                        client_count
                    );

                    if client_count == 0 {
                        let _ = state.qu_status.send(
                            QuStatus::Disconnected
                        );
                    }
                }

                Some(QuCommand::Change(change)) => {
                    eprintln!(
                        "[QU] Cannot send change; Qu is not connected: {:?}",
                        change
                    );
                }

                None => break,
            }

            continue;
        }

        /*
         * At this point both:
         *
         *   - at least one WebSocket client exists
         *   - a Qu TCP connection exists
         *
         * Handle the two directions concurrently.
         */
        let qu_stream = stream
            .as_mut()
            .expect("stream checked above");

        tokio::select! {
            result = qu_stream.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        println!("[QU] Connection closed");

                        stream = None;

                        let _ = state.qu_status.send(
                            QuStatus::Error {
                                message:
                                    "Qu connection closed"
                                        .to_string(),
                            },
                        );
                    }

                    Ok(bytes_read) => {
                        let events =
                            parser.push(&buffer[..bytes_read]);

                        for event in events {
                            let synchronised =
                                handler.handle(event).await;

                            if synchronised {
                                let (name, address) =
                                    match &*state.qu_status.borrow() {
                                        QuStatus::Connecting {
                                            name,
                                            address,
                                        } => (
                                            name.clone(),
                                            address.clone(),
                                        ),

                                        QuStatus::Synchronising => {
                                            (
                                                "unknown".to_string(),
                                                "unknown".to_string(),
                                            )
                                        }

                                        QuStatus::Connected {
                                            name,
                                            address,
                                        } => (
                                            name.clone(),
                                            address.clone(),
                                        ),

                                        _ => {
                                            (
                                                "unknown".to_string(),
                                                "unknown".to_string(),
                                            )
                                        }
                                    };

                                println!(
                                    "[QU] Synchronisation complete"
                                );

                                let _ = state.qu_status.send(
                                    QuStatus::Connected {
                                        name,
                                        address,
                                    }
                                );
                            }
                        }
                    }

                    Err(error) => {
                        eprintln!(
                            "[QU] Read error: {}",
                            error
                        );

                        stream = None;

                        let _ = state.qu_status.send(
                            QuStatus::Error {
                                message: error.to_string(),
                            },
                        );
                    }
                }
            }

            command = commands.recv() => {
                match command {
                    Some(QuCommand::ClientConnected) => {
                        client_count += 1;

                        println!(
                            "[CLIENT] Connected; {} client(s)",
                            client_count
                        );
                    }

                    Some(QuCommand::ClientDisconnected) => {
                        client_count =
                            client_count.saturating_sub(1);

                        println!(
                            "[CLIENT] Disconnected; {} client(s)",
                            client_count
                        );

                        if client_count == 0 {
                            println!(
                                "[QU] Last client disconnected; \
                                 closing Qu connection"
                            );

                            stream = None;

                            let _ = state.qu_status.send(
                                QuStatus::Disconnected
                            );

                            /*
                             * Reset the authoritative state. The next
                             * Qu connection will repopulate it.
                             */
                            let mut mixer =
                                state.mixer.write().await;

                            *mixer = MixerState::default();

                            parser =
                                qu::parser::Parser::new();
                        }
                    }

                    Some(QuCommand::Change(change)) => {
                        if let Err(error) =
                            send_change_to_qu(
                                qu_stream,
                                &state,
                                &change,
                            )
                            .await
                        {
                            eprintln!(
                                "[QU TX] Failed to send {:?}: {}",
                                change,
                                error
                            );

                            stream = None;

                            let _ = state.qu_status.send(
                                QuStatus::Error {
                                    message: error.to_string(),
                                },
                            );
                        } else if let Err(error) =
                            request_system_state(
                                qu_stream
                            )
                            .await
                        {
                            eprintln!(
                                "[QU TX] Failed to request state \
                                 after {:?}: {}",
                                change,
                                error
                            );

                            stream = None;

                            let _ = state.qu_status.send(
                                QuStatus::Error {
                                    message: error.to_string(),
                                },
                            );
                        }
                    }

                    None => {
                        println!(
                            "[QU] Command channel closed"
                        );

                        stream = None;
                        break;
                    }
                }
            }
        }
    }

    println!("[QU] Manager stopped");
}

async fn connect_and_synchronise(
    state: &AppState,
    stream: &mut Option<TcpStream>,
    parser: &mut qu::parser::Parser,
    buffer: &mut [u8; 4096],
    handler: &QuHandler,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = state.qu_status.send(QuStatus::Discovering);

    let (mut qu_stream, name) =
        discover_and_connect(state).await?;

    let address = qu_stream
        .peer_addr()
        .map(|address| address.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let _ = state.qu_status.send(
        QuStatus::Synchronising
    );

    request_system_state(&mut qu_stream).await?;

    /*
     * A fresh parser is required for every fresh TCP connection.
     */
    *parser = qu::parser::Parser::new();

    loop {
        let bytes_read = qu_stream.read(buffer).await?;

        if bytes_read == 0 {
            return Err("Qu connection closed during synchronisation".into());
        }

        let events = parser.push(&buffer[..bytes_read]);

        for event in events {
            if handler.handle(event).await {
                println!(
                    "[QU] Initial system state received"
                );

                let _ = state.qu_status.send(
                    QuStatus::Connected {
                        name: name.clone(),
                        address: address.clone(),
                    }
                );

                *stream = Some(qu_stream);

                return Ok(());
            }
        }
    }
}

async fn websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    println!("[CLIENT] WebSocket upgrade requested");

    ws.on_upgrade(move |socket| {
        handle_websocket(socket, state)
    })
}

async fn handle_websocket(
    socket: WebSocket,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();

    let handler = ClientHandler::new(state.clone());

    /*
     * Subscribe before announcing ourselves to the Qu manager. This avoids
     * missing a status transition caused by our own connection.
     */
    let mut qu_status = state.qu_status.subscribe();
    let mut client_updates = state.client_updates.subscribe();

    if state
        .qu_commands
        .send(QuCommand::ClientConnected)
        .await
        .is_err()
    {
        eprintln!(
            "[CLIENT] Failed to notify Qu manager"
        );
        return;
    }

    println!("[CLIENT] Connected");

    /*
     * Send the current status immediately, then wait for status changes.
     */
    let status = qu_status.borrow().clone();
    if send_status(
        &mut sender,
        &status,
    )
    .await
    .is_err()
    {
        let _ = state
            .qu_commands
            .send(QuCommand::ClientDisconnected)
            .await;

        return;
    }

    /*
     * If the Qu is already connected when this client arrives, send the
     * authoritative state immediately.
     *
     * Otherwise the state will be sent when QuStatus::Connected arrives.
     */
    if matches!(
        &*qu_status.borrow(),
        QuStatus::Connected { .. }
    ) {
        if send_current_state(
            &mut sender,
            &state,
        )
        .await
        .is_err()
        {
            let _ = state
                .qu_commands
                .send(QuCommand::ClientDisconnected)
                .await;

            return;
        }
    }

    loop {
        tokio::select! {
            result = receiver.next() => {
                let Some(result) = result else {
                    println!("[CLIENT] Disconnected");
                    break;
                };

                let Ok(message) = result else {
                    eprintln!(
                        "[RX UI] WebSocket receive error"
                    );
                    break;
                };

                let Message::Text(text) = message else {
                    continue;
                };

                handler.handle(&text).await;
            }

            result = qu_status.changed() => {
                if result.is_err() {
                    break;
                }

                let status = qu_status.borrow().clone();

                println!(
                    "[TX UI] Qu status: {:?}",
                    status
                );

                if send_status(
                    &mut sender,
                    &status,
                )
                .await
                .is_err()
                {
                    eprintln!(
                        "[UI] Client disconnected"
                    );
                    break;
                }

                /*
                 * The state is only authoritative once the Qu has completed
                 * its initial synchronisation.
                 */
                if matches!(
                    status,
                    QuStatus::Connected { .. }
                ) {
                    if send_current_state(
                        &mut sender,
                        &state,
                    )
                    .await
                    .is_err()
                    {
                        eprintln!(
                            "[UI] Client disconnected"
                        );
                        break;
                    }
                }
            }

            result = client_updates.recv() => {
                let Ok(change) = result else {
                    continue;
                };

                /*
                 * A change received from the Qu is only meaningful to this
                 * client while the Qu is connected.
                 */
                if !matches!(
                    &*qu_status.borrow(),
                    QuStatus::Connected { .. }
                ) {
                    continue;
                }

                let message =
                    ServerMessage::Change(change);

                if sender
                    .send(Message::Text(
                        match serde_json::to_string(&message) {
                            Ok(message) => message.into(),
                            Err(error) => {
                                eprintln!(
                                    "[TX UI] Failed to serialise \
                                     mixer change: {}",
                                    error
                                );
                                continue;
                            }
                        }
                    ))
                    .await
                    .is_err()
                {
                    eprintln!(
                        "[UI] Client disconnected"
                    );
                    break;
                }
            }
        }
    }

    /*
     * This client is no longer interested in the Qu. The manager will close
     * the TCP connection if this was the final client.
     */
    if state
        .qu_commands
        .send(QuCommand::ClientDisconnected)
        .await
        .is_err()
    {
        eprintln!(
            "[CLIENT] Failed to notify Qu manager of disconnect"
        );
    }

    println!("[CLIENT] Connection closed");
}

async fn send_status(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    status: &QuStatus,
) -> Result<(), axum::Error> {
    let message = ServerMessage::QuStatus {
        status: status.clone(),
    };

    let text = serde_json::to_string(&message)
        .map_err(|_| axum::Error::new(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to serialise Qu status",
            )
        ))?;

    sender
        .send(Message::Text(text.into()))
        .await
}

async fn send_current_state(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
) -> Result<(), axum::Error> {
    let mixer = state.mixer.read().await.clone();

    let message = ServerMessage::State(mixer);

    let text = serde_json::to_string(&message)
        .map_err(|_| axum::Error::new(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to serialise mixer state",
            )
        ))?;

    sender
        .send(Message::Text(text.into()))
        .await
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client_updates, _) =
        broadcast::channel(64);

    let (qu_commands, qu_command_receiver) =
        mpsc::channel(64);

    let (qu_status, _) =
        watch::channel(QuStatus::Disconnected);

    let state = AppState {
        mixer: Arc::new(
            RwLock::new(
                MixerState::default()
            )
        ),
        client_updates,
        qu_commands,
        qu_status,
    };

    let qu_state = state.clone();

    tokio::spawn(async move {
        run_qu_manager(
            qu_state,
            qu_command_receiver,
        )
        .await;
    });

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                "thwip server"
            }),
        )
        .route(
            "/ws",
            get(websocket),
        )
        .with_state(state);

    let address: SocketAddr =
        "0.0.0.0:3000".parse()?;

    let listener =
        TcpListener::bind(address).await?;

    println!(
        "[SERVER] Listening on {}",
        address
    );

    axum::serve(
        listener,
        app,
    )
    .await?;

    Ok(())
}
