# Quip

> **quip** /kwɪp/ *verb*  
> to make a humorous and clever remark.

`quip` is a communication library for connecting to Qu mixers over a LAN. The name deliberately combines **Qu** and **IP** — describing communication with a Qu mixer over an IP network — forming the English word *quip*, quite fittingly, for something communicated from one party to another.

`qu` is the underlying protocol library, handling the encoding and decoding of raw protocol data.

`quippi` is a proxy for connecting multiple `quip` clients to a single Qu mixer over TCP (one-to-many). Designed to run as an always-on network service, such as on a Raspberry Pi.

> **squib** /skwɪb/ *noun*  
> a brief satirical or witty piece of writing or speech, or a short, sometimes humorous piece in a newspaper or magazine, used as a filler.

`squib` is an emulator of a Qu mixer device, for development and testing without the physical hardware.

## Apps
| `./apps/...` | Description |
|--------------|-------------|
| `quip-web`   | Controller web app for Qu devices |
<!-- very tempting to call the "web" version of "quip": "thwip" in honour of Spider-Man... -->
<!-- https://static0.srcdn.com/wordpress/wp-content/uploads/2021/10/Spider-Man-Thwip.jpg -->

```bash
bun run dev
```
```bash
bun run build
```

## Crates
| `./crates/...` | Description |
|----------------|-------------|
| `qu`           | [Qu Mixer MIDI Protocol (Firmware V1.9+)](./Qu_MIDI_Protocol_V1.9.pdf) Library |
| `quip`         | Qu Mixer LAN (TCP) Client Communication Library |
| `quippi`       | Qu Proxy Connector |
| `squib`        | Qu Mixer Emulator |

```bash
cargo check --workspace
cargo test --workspace
```
```bash
# squib
sudo ufw allow 51325/tcp
sudo ufw allow 51320/udp
cargo run -p squib -- 0.0.0.0:51325
```
```bash
# quippi
cargo run -p quippi
```
```bash
cargo build --workspace --release
```

## Packages
| `./packages/...` | Description |
|------------------|-------------|
| `quip`           | Qu Mixer LAN (TCP) Client Communication Library |

```bash
bun run build
```
