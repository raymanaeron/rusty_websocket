# Rust WebSocket Server and Client Framework

This project provides a complete WebSocket-based pub-sub (publish-subscribe) messaging system implemented in Rust. It consists of:

- A WebSocket server using `axum`
- A reusable WebSocket client (`WsClient`)
- Test clients in both Rust and JavaScript (browser)
- A static HTML+JS interface for running client tests
- A configurable launcher for local or web test modes

---

## How to Build and Run

### Build

To build the server:

```sh
cargo build
```

Or for release:

```sh
cargo build --release
```

### Run

There are two modes for running:

#### 1. Local Rust Test Mode

```sh
./server
```

This mode:
- Launches the WebSocket server on `ws://127.0.0.1:8081/ws`
- Runs the Rust-based `client_tests.rs` test suite

#### 2. Web UI Test Mode

```sh
./server --web
```

This mode:
- Starts the WebSocket server on port `8081`
- Serves the static HTML+JS web client from `target/debug/web/` on `http://localhost:8080`

To run the web test:
1. Launch the server with `--web`
2. Open your browser to [http://localhost:8080](http://localhost:8080)
3. Click “Start Test” to trigger the same pub-sub behavior using JavaScript clients

> Ensure your working directory is `target/debug` so static file serving works correctly.

---

## WebSocket Server

The server is defined in `lib.rs` and listens for clients to:

- `register-name:<ClientName>` — identifies the client by name
- `subscribe:<Topic>` — subscribes to a topic
- `unsubscribe:<Topic>` — unsubscribes from a topic
- `publish:<Topic>:<Message>` — broadcasts to subscribers of a topic

### Client Cleanup

On disconnect, clients are unsubscribed automatically from all topics.

---

## WebSocket Client (`ws_client.rs`)

### Exposed Methods

```rust
pub struct WsClient { ... }

impl WsClient {
    pub async fn connect(name: &str, url: &str) -> Result<Self>;
    pub async fn subscribe(&mut self, topic: &str);
    pub async fn unsubscribe(&mut self, topic: &str);
    pub async fn publish(&mut self, topic: &str, payload: &str);
}
```

Each `WsClient` connects to the server, registers its name, and provides simple pub-sub APIs.

---

## Rust Test Client (`client_tests.rs`)

```rust
pub async fn run_client_tests()
```

Simulates 3 clients with the following logic:

- **Client1** subscribes to `DetectCustomerEvent`, `NetworkConnectedEvent`, then publishes to `RegistrationCompleteEvent`
- **Client2** subscribes to `DetectCustomerEvent`, `RegistrationCompleteEvent`, then publishes to `NetworkConnectedEvent`
- **Client3** subscribes to `NetworkConnectedEvent`, `RegistrationCompleteEvent`, then publishes to `DetectCustomerEvent`

This demonstrates correct fan-out of messages to appropriate subscribers.

---

## Web Test Client (`index.html`, `tests.js`)

### `index.html`

A simple page with a “Start Test” button and a scrollable log area.

### `tests.js`

Defines a `WsClientJS` that mirrors the Rust client and runs the same test pattern on button click:

- Each client logs its subscription and received messages to the scroll area
- Confirms full cross-client delivery like the Rust test

---

## Folder Structure

```
.
├── src/
│   ├── main.rs
│   ├── client_tests.rs
│   └── ws_client.rs
├── web/
│   ├── index.html
│   └── tests.js
├── target/debug/server.exe
├── target/debug/web/...
```

---

## Notes

- Requires Rust 2021+ and `tokio`, `axum`, `futures-util`, `tower-http`
- All logs are printed to console or appended to the HTML text area