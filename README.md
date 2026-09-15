# quip

| Crate   | Description |
|---------|-------------|
| `qu`    | [Qu Mixer MIDI Protocol (Firmware V1.9+)](./Qu_MIDI_Protocol_V1.9.pdf) Library |
| `quip`  | Qu Mixer LAN (TCP) Client Communication Library |
| `squib` | Qu Mixer Emulator |

```bash
cargo check --workspace
```
```bash
cargo build --workspace
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
