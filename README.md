# rust-ws

A simple WebSocket server in Rust using [warp](https://crates.io/crates/warp) and [tokio](https://crates.io/crates/tokio).

## Features
- WebSocket endpoint (`/ws`) for real-time communication
- Private WebSocket endpoint (`/ws-private`) with token-based authentication
- Health check endpoint (`/health-check`)
- JSON-based message protocol
- CORS enabled for all origins
- Structured error handling

## Endpoints

### Health Check
- **GET** `/health-check`
- Returns: `Server is running!`

### Public WebSocket
- **WS** `/ws`
- Accepts WebSocket connections from any client.
- Example message to send:
  ```json
  {"kind": "test", "token": "6smtr8ke3s7yq63f3zug9z3th"}
  ```
- Example response:
  ```json
  {"status": "success", "response": "awesome message"}
  ```

### Private WebSocket
- **WS** `/ws-private`
- Requires `Authorization` header: `Token 6smtr8ke3s7yq63f3zug9z3th`
- Example using [websocat](https://github.com/vi/websocat):
  ```sh
  websocat -H="Authorization: Token 6smtr8ke3s7yq63f3zug9z3th" ws://127.0.0.1:7878/ws-private
  ```

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (edition 2021 or later)

### Build and Run
```sh
cd rust-ws
cargo run
```

The server will start on `127.0.0.1:7878` by default. Use `--port` to specify a different port:
```sh
cargo run -- --port 9000
```

## Testing Endpoints

### Health Check
```sh
curl http://127.0.0.1:7878/health-check
```

### WebSocket (public)
```sh
websocat ws://127.0.0.1:7878/ws
```

### WebSocket (private)
```sh
websocat -H="Authorization: Token 6smtr8ke3s7yq63f3zug9z3th" ws://127.0.0.1:7878/ws-private
```

