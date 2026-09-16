# Quip

> **quip** /kwɪp/ *verb*  
> to make a humorous and clever remark.

`quip` is a communication library for connecting to Qu mixers over a LAN. The name deliberately combines **Qu** and **IP** — describing communication with a Qu mixer over an IP network — forming the English word *quip*, quite fittingly, for something communicated from one party to another.

`qu` is the underlying protocol library, handling the encoding and decoding of raw protocol data.

> **squib** /skwɪb/ *noun*  
> a brief satirical or witty piece of writing or speech, or a short, sometimes humorous piece in a newspaper or magazine, used as a filler.

`squib` is an emulator of a Qu mixer device, for development and testing without the physical hardware.

## Apps
| App        | Description |
|------------|-------------|
| `quip-web` | Controller web app for Qu devices |

```bash
bun run dev
```
```bash
bun run build
```

## Crates
| Crate   | Description |
|---------|-------------|
| `qu`    | [Qu Mixer MIDI Protocol (Firmware V1.9+)](./Qu_MIDI_Protocol_V1.9.pdf) Library |
| `quip`  | Qu Mixer LAN (TCP) Client Communication Library |
| `squib` | Qu Mixer Emulator |

```bash
cargo check --workspace
```
```bash
cargo test --workspace
```
```bash
cargo run -p squib -- 0.0.0.0:51325
```
```bash
cargo build --workspace --release
```

## Packages
| Package | Description |
|---------|-------------|
| `quip`  | Qu Mixer LAN (TCP) Client Communication Library |

```bash
bun run build
```
