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
    net::TcpStream,
    sync::{
        broadcast,
        RwLock,
    },
};

use qu::{messages::QuEvent, protocol::TCP_PORT};

use quip::{
    qu_to_state,
    qu_from_state,
    state::MixerState,
    client::discovery::discover,
};

#[derive(Clone)]
struct AppState {
    mixer: Arc<RwLock<MixerState>>,
    client_updates: broadcast::Sender<MixerChange>,
    qu_updates: broadcast::Sender<MixerChange>,
    state_ready: tokio::sync::watch::Sender<bool>,
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", content = "number")]
pub enum ChannelRef {
    Input(u8),
    Stereo(u8),
    Mix(u8),
    Lr,
    MuteGroup(u8),
}

impl ChannelRef {
    fn channel(self) -> Option<qu::channels::Channel> {
        use qu::channels::Channel;

        match self {
            Self::Input(number) => Channel::input(number),
            Self::Stereo(number) => Channel::stereo(number),
            Self::Mix(number) => Channel::mix(number),
            Self::Lr => Some(Channel::lr()),
            Self::MuteGroup(number) => Channel::mute_group(number),
        }
    }

    fn from_channel(channel: qu::channels::Channel) -> Option<Self> {
        use qu::channels::Channel;

        // CH
        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(Self::Input(number));
            }
        }

        // ST
        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(Self::Stereo(number));
            }
        }

        // MIX
        for number in 1..=8 {
            if Channel::mix(number) == Some(channel) {
                return Some(Self::Mix(number));
            }
        }
        if Channel::lr() == channel {
            return Some(Self::Lr);
        }

        // MG
        for number in 1..=4 {
            if Channel::mute_group(number) == Some(channel) {
                return Some(Self::MuteGroup(number));
            }
        }

        None
    }
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

        // Relay request to Qu
        if self.state.qu_updates.send(change.clone()).is_err() {
            println!("[TX QU] No clients connected");
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

    async fn handle(&self, event: QuEvent) {
        if !matches!(event, QuEvent::ActiveSense) {
            println!("[RX QU] {:?}", event);
        }

        // Derive the client-facing change before giving ownership of the
        // event to quip's event handler.
        let change = match &event {
            QuEvent::Fader { channel, value } => {
                let Some(channel) = ChannelRef::from_channel(*channel) else {
                    eprintln!(
                        "[RX QU] Unsupported fader channel: {:?}",
                        channel
                    );
                    return;
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
                    return;
                };

                Some(MixerChange::Mute {
                    channel,
                    muted: *muted,
                })
            }

            QuEvent::ActiveSense => None,

            _ => None,
        };

        // quip owns the authoritative interpretation of Qu events.
        {
            let mut mixer = self.state.mixer.write().await;
            qu_to_state::handle_event(&mut mixer, event.clone());
        }

        // The initial GET_SYSTEM_STATE response has finished.
        //
        // Replace `SystemStateComplete` with the actual QuEvent emitted by
        // the protocol when the response is complete.
        let is_end_sync = matches!(
            &event,
            QuEvent::SysEx(data)
                if data.as_slice() == qu::protocol::end_sync()
        );
        if let QuEvent::SysEx(data) = &event {
            println!("[QU] SysEx received: {:02X?}", data);
            println!("[QU] SysEx expected: {:02X?}", qu::protocol::end_sync());
        }
        if is_end_sync {
            println!("[QU] Initial system state received");
            let _ = self.state.state_ready.send_replace(true);
            return;
        }

        // Do not send individual state changes to clients while the initial
        // system state is still being populated. The client will receive the
        // complete authoritative state once state_ready becomes true.
        if let Some(change) = change {
            if *self.state.state_ready.borrow() {
                println!("[STATE] Applied {:?}", change);

                if self.state.client_updates.send(change.clone()).is_err() {
                    println!("[QU TX → CLIENT] No clients connected");
                } else {
                    println!("[QU TX → CLIENT] {:?}", change);
                }
            }
        }
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

async fn run_qu(
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // Discovery uses UDP port 51320, whereas Qu mixer control uses
    // TCP port 51325. Keep the discovered IP but use the TCP port.
    let address = SocketAddr::new(
        device.address.ip(),
        TCP_PORT,
    );

    println!("[QU] Connecting to {}", address);

    let mut stream = TcpStream::connect(address).await?;

    println!("[QU] Connected");

    // Ask the Qu for its current system state.
    request_system_state(&mut stream).await?;

    let mut parser = qu::parser::Parser::new();
    let mut buffer = [0u8; 4096];

    let mut qu_updates = state.qu_updates.subscribe();

    let handler = QuHandler::new(state.clone());

    loop {
        tokio::select! {
            result = stream.read(&mut buffer) => {
                let bytes_read = result?;

                if bytes_read == 0 {
                    println!("[QU] Connection closed");
                    break;
                }

                let events = parser.push(&buffer[..bytes_read]);

                for event in events {
                    handler.handle(event).await;
                }
            }

            result = qu_updates.recv() => {
                let Ok(change) = result else {
                    continue;
                };

                if let Err(error) = send_change_to_qu(
                    &mut stream,
                    &state,
                    &change,
                ).await {
                    eprintln!(
                        "[QU TX] Failed to send {:?}: {}",
                        change,
                        error
                    );
                    break;
                }

                if let Err(error) = request_system_state(&mut stream).await {
                    eprintln!(
                        "[QU TX] Failed to request state after {:?}: {}",
                        change,
                        error
                    );
                    break;
                }
            }
        }
    }

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

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client_updates, _) = broadcast::channel(64);
    let (qu_updates, _) = broadcast::channel(64);
    let (state_ready, _) = tokio::sync::watch::channel(false);

    let state = AppState {
        mixer: Arc::new(RwLock::new(MixerState::default())),
        client_updates,
        qu_updates,
        state_ready,
    };

    let app = Router::new()
        .route("/", get(|| async { "thwip server" }))
        .route("/ws", get(websocket))
        .with_state(state.clone());

    let address: SocketAddr = "0.0.0.0:3000".parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("[SERVER] Listening on {}", address);

    let qu_state = state.clone();

    tokio::spawn(async move {
        if let Err(error) = run_qu(qu_state).await {
            eprintln!("[QU] Connection error: {}", error);
        }
    });

    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    println!("[CLIENT] WebSocket upgrade requested");

    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(
    socket: WebSocket,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut client_updates = state.client_updates.subscribe();

    let handler = ClientHandler::new(state.clone());

    println!("[CLIENT] Connected");

    // Send the authoritative state when the client connects.
    let mut state_ready = state.state_ready.subscribe();
    while !*state_ready.borrow() {
        if state_ready.changed().await.is_err() {
            return;
        }
    }
    // The Qu has now populated the authoritative state.
    let current_state = state.mixer.read().await.clone();

    let Ok(message) = serde_json::to_string(&current_state) else {
        eprintln!("[TX UI] Failed to serialise mixer state");
        return;
    };

    if sender
        .send(Message::Text(message.into()))
        .await
        .is_err()
    {
        eprintln!("[TX UI] Client disconnected");
        return;
    }

    loop {
        tokio::select! {
            result = receiver.next() => {
                let Some(result) = result else {
                    println!("[UI] Disconnected");
                    break;
                };

                let Ok(message) = result else {
                    eprintln!("[RX UI] WebSocket receive error");
                    break;
                };

                let Message::Text(text) = message else {
                    continue;
                };

                handler.handle(&text).await;
            }

            result = client_updates.recv() => {
                let Ok(change) = result else {
                    continue;
                };

                let Ok(message) = serde_json::to_string(&change) else {
                    eprintln!(
                        "[TX UI] Failed to serialise {:?}",
                        change
                    );
                    continue;
                };

                println!("[TX UI] {}", message);

                if sender
                    .send(Message::Text(message.into()))
                    .await
                    .is_err()
                {
                    eprintln!("[UI] Client disconnected");
                    break;
                }
            }
        }
    }

    println!("[UI] Connection closed");
}
