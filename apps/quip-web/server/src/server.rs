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

use qu::messages::QuEvent;

use quip::{
    qu_to_state,
    qu_from_state,
    state::MixerState,
};

#[derive(Clone)]
struct AppState {
    mixer: Arc<RwLock<MixerState>>,
    client_updates: broadcast::Sender<MixerChange>,
    qu_updates: broadcast::Sender<MixerChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MixerChange {
    Fader {
        channel: ChannelRef,
        value: f32,
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
}

impl ChannelRef {
    fn channel(self) -> Option<qu::channels::Channel> {
        use qu::channels::Channel;

        match self {
            Self::Input(number) => Channel::input(number),
            Self::Stereo(number) => Channel::stereo(number),
            Self::Mix(number) => Channel::mix(number),
            Self::Lr => Some(Channel::lr()),
        }
    }

    fn from_channel(channel: qu::channels::Channel) -> Option<Self> {
        use qu::channels::Channel;

        for number in 1..=16 {
            if Channel::input(number) == Some(channel) {
                return Some(Self::Input(number));
            }
        }

        for number in 1..=3 {
            if Channel::stereo(number) == Some(channel) {
                return Some(Self::Stereo(number));
            }
        }

        for number in 1..=10 {
            if Channel::mix(number) == Some(channel) {
                return Some(Self::Mix(number));
            }
        }

        if Channel::lr() == channel {
            return Some(Self::Lr);
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
        if self.state.client_updates.send(change.clone()).is_err() {
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

                Some(MixerChange::Fader {
                    channel,
                    value: qu::parameters::fader_to_db(*value),
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
            qu_to_state::handle_event(&mut mixer, event);
        }

        if let Some(change) = change {
            println!("[STATE] Applied {:?}", change);

            // A Qu-originated change is sent towards clients.
            if self.state.client_updates.send(change.clone()).is_err() {
                println!("[QU TX → CLIENT] No clients connected");
            } else {
                println!("[QU TX → CLIENT] {:?}", change);
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
                mixer.set_fader(channel, *value);
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
    let address = "127.0.0.1:51325";

    println!("[QU] Connecting to {}", address);

    let mut stream = TcpStream::connect(address).await?;

    println!("[QU] Connected");

    // Ask the Qu for its current system state.
    stream
        .write_all(&qu::protocol::get_system_state())
        .await?;

    println!("[QU TX] GET_SYSTEM_STATE");

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
            }
        }
    }

    Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client_updates, _) = broadcast::channel(64);
    let (qu_updates, _) = broadcast::channel(64);

    let state = AppState {
        mixer: Arc::new(RwLock::new(MixerState::default())),
        client_updates,
        qu_updates,
    };

    let app = Router::new()
        .route("/", get(|| async { "quip-web server" }))
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
    let current_state = state.mixer.read().await.clone();

    let Ok(message) = serde_json::to_string(&current_state) else {
        eprintln!("[TX UI] Failed to serialise mixer state");
        return;
    };

    println!("[TX UI] Initial mixer state");

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
                        "[CLIENT TX] Failed to serialise {:?}",
                        change
                    );
                    continue;
                };

                println!("[CLIENT TX] {}", message);

                if sender
                    .send(Message::Text(message.into()))
                    .await
                    .is_err()
                {
                    eprintln!("[CLIENT TX] Client disconnected");
                    break;
                }
            }
        }
    }

    println!("[CLIENT] Connection closed");
}
