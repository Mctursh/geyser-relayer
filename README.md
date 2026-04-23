# geyser-relay

A WebSocket relay that bridges a Solana Geyser gRPC stream to browser clients.

Subscribes to a [geyser-plugin](https://github.com/mctursh/geyser-plugin) plugin's gRPC stream and fans out account updates to connected WebSocket clients.

## Architecture

Geyser plugin (gRPC :50051) → Relay (gRPC client → broadcast → WebSocket :3000) → Browsers


## Run

Start the validator with the Geyser plugin first, then:

```bash
cargo run
```

## Connect

WebSocket endpoint: `ws://0.0.0.0:3000/ws`
