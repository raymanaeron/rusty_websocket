# WebSocket Messaging Bus for Disconnected Apps

This project implements a lightweight pub-sub (publish-subscribe) messaging framework that allows multiple disconnected apps—written in Rust or JavaScript—to talk to each other in real time. It is ideal for system orchestration, onboarding flows, or multi-client coordination using a WebSocket-based messaging bus.

The framework includes:
- A high-performance WebSocket server written in Rust with `axum`
- A pluggable Rust WebSocket client with callback-based message handling
- JSON-based message structure including topic, payload, publisher, and timestamp
- A JS/WebSocket interface for browser clients
- Full logging and test harness in both Rust and browser environments

---

## 🧩 Architecture Overview

The architecture follows a centralized **message bus model**:

```
       ┌────────────┐
       │ JS Client  │
       └────┬───────┘
            │
       ┌────▼────┐
       │ WebSocket│
       │  Server │  ◄────────────┐
       └────┬────┘               │
            │                    │
  ┌─────────▼──────────┐ ┌───────▼────────┐
  │ Rust Client (CLI)  │ │ Rust Client UI │
  └────────────────────┘ └────────────────┘
```

Each client (Rust or JS) registers itself by name, subscribes to one or more topics, and optionally publishes messages to those topics. Subscribers automatically receive published messages in real-time using a shared JSON protocol.

---

## How It Works

### Message Protocol

All messages follow a JSON structure:

```json
{
  "publisher_name": "Client1",
  "topic": "NetworkConnectedEvent",
  "payload": "Network connected",
  "timestamp": "2025-04-24T01:25:37Z"
}
```

### Supported Commands

- `register-name:<ClientName>` — identifies a client
- `subscribe:<Topic>` — subscribes to a topic
- `unsubscribe:<Topic>` — removes subscription
- `publish-json:<JSON>` — sends a JSON payload to all topic subscribers

---

## Running and Testing

### Build the Server

```sh
cargo build
```

Add `chrono = { version = "0.4", features = ["serde", "alloc"] }` to your `Cargo.toml` if not already present.

### Test Mode 1: Rust Clients (CLI)

```sh
cargo run
```

This mode:
- Starts the WebSocket server on `ws://127.0.0.1:8081/ws`
- Automatically launches 3 clients via `client_tests.rs`
- Each client subscribes and publishes using `WsClient` with timestamps and topic routing

### Test Mode 2: Browser Clients

```sh
cargo run -- server --web
```

Then open your browser to [http://localhost:8080](http://localhost:8080)

- Static HTML/JS served on port 8080
- WebSocket server on port 8081
- Click “Start Test” to connect and simulate 3 JS clients using `tests.js`

> Ensure you run from the directory containing the `web/` folder or configure path accordingly.

---

## 🧠 Rust Client API

```rust
let mut client = WsClient::connect("Client1", "ws://127.0.0.1:8081/ws").await?;
client.subscribe("Client1", "TopicA", "payload").await;
client.publish("Client1", "TopicA", "Hello World", &Utc::now().to_rfc3339()).await;
client.on_message("TopicA", |msg| println!("Received: {}", msg));
```

### Method Signatures

```rust
pub async fn connect(client_name: &str, ws_url: &str) -> Result<Self>;
pub async fn subscribe(&mut self, subscriber_name: &str, topic: &str, payload: &str);
pub async fn publish(&mut self, publisher_name: &str, topic: &str, payload: &str, timestamp: &str);
pub fn on_message(&mut self, topic: &str, callback: impl Fn(String) + Send + Sync + 'static);
```

---

## JavaScript Client Flow

- `createClient(name, url, topics, publishAction)` registers a name, subscribes to topics, and sends a structured JSON message
- Incoming messages are decoded and logged with publisher, topic, payload, and timestamp

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
├── lib.rs
├── Cargo.toml
└── README.md
```

---

## 🔧 Dependencies

- Rust 2021+
- `tokio`, `axum`, `tower-http`, `futures-util`, `serde`, `chrono`
- WebSocket support in modern browsers

---

## 📌 Summary

This project enables coordinated orchestration between UI, CLI, and embedded Rust or JS apps through a shared, topic-based WebSocket message bus. Ideal for workflows like device setup, distributed tests, or system integration.
