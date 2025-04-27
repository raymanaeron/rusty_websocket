# TODO: THIS IS A WORK IN PROGRESS
- The encryption/decryption working ok but not hooked up with the web socket server. This is a tough one and I need to pick it up some other time
- The web socket server has session based pub/sub implmentation which are hooked up and working properly

# WebSocket-based Pub/Sub Framework for Distributed Applications

## Problem Statement
Modern applications often consist of multiple distributed components that need to communicate in real-time. Traditional HTTP-based communication can be cumbersome and doesn't support real-time updates efficiently. Additionally, managing state and coordination between multiple clients (web, desktop, CLI) becomes complex without a centralized messaging system.

When multiple users or devices connect to the same messaging system, there's a critical need for session isolation. Without proper session boundaries, messages intended for one user's session might be delivered to another user's session, creating privacy concerns and data leakage. This is especially problematic in multi-tenant applications where different user sessions must remain strictly isolated.

## Solution
This framework provides a lightweight pub/sub (publish-subscribe) messaging system using WebSocket technology. It includes:
- A high-performance Rust WebSocket server using axum
- A Rust client library with async support and error handling
- A JavaScript client implementation for web browsers
- JSON-based message protocol for cross-platform compatibility
- Session-based message routing to ensure privacy and data isolation between users

## Benefits
- Real-time bidirectional communication
- Language-agnostic messaging (Rust, JavaScript)
- Topic-based message routing
- Connection state management and error handling
- Simple API for both Rust and JavaScript clients
- Session-scoped messaging to prevent cross-session data leakage
- Proper isolation between different user sessions or application instances

## Architecture

```
[Rust Client] ←→ [WebSocket Server] ←→ [Browser Client]
     ↑               ↑                        ↑
     |               |                        |
   connect()     JSON Message              connect()
   subscribe()    Protocol              subscribe()
   publish()                            publish()
```

### Message Protocol
```json
{
  "publisher_name": "Client1",
  "topic": "NetworkConnectedEvent",
  "payload": "Network connected",
  "timestamp": "2024-01-24T10:25:37Z",
  "session_id": "session-user123"
}
```

## Using the Rust Client

### Connection
```rust
// Connect with default session ID (derived from client name)
let mut client = WsClient::connect("Client1", "ws://127.0.0.1:8081/ws").await?;

// Or connect with a specific session ID
let mut client = WsClient::connect_with_session("Client1", "user-session-123", "ws://127.0.0.1:8081/ws").await?;
```

### Subscribe to Topics
```rust
// Subscribe to multiple topics within the client's session
client.subscribe("Client1", "DetectCustomerEvent", "no-payload").await;
client.subscribe("Client1", "NetworkConnectedEvent", "no-payload").await;

// Register message handlers
// Messages will only be received if published to the same session
client.on_message("DetectCustomerEvent", move |msg| {
    println!("Customer Event: {}", msg);
});
```

### Publishing Messages
```rust
use chrono::Utc;

// Publish with timestamp to the client's session
// Only subscribers within the same session will receive this message
let result = client.publish(
    "Client1",
    "NetworkConnectedEvent",
    "Network connected successfully",
    &Utc::now().to_rfc3339()
).await;

if let Err(e) = result {
    println!("Failed to publish: {}", e);
}
```

## Using the JavaScript Client

### Connection and Subscribe
```javascript
// Connect with a specific session ID
const client = await createClient(
    "WebClient1",
    "ws://localhost:8081/ws",
    {
        subscriptions: ["DetectCustomerEvent", "NetworkConnectedEvent"]
    },
    // Optional publish action
    {
        topic: "RegistrationEvent",
        message: "Web client registered"
    },
    // Session ID (optional, defaults to "session-WebClient1")
    "user-session-456"
);

// Message handler is automatically set up in createClient
// Only messages published to "user-session-456" will be received
```

### Publishing Messages
```javascript
const message = {
    publisher_name: clientName,
    topic: "StatusEvent",
    payload: "Status updated",
    timestamp: new Date().toISOString(),
    session_id: "user-session-456"  // Specify the session scope
};

ws.send(`publish-json:${JSON.stringify(message)}`);
```

## Running the Framework

### CLI Mode (Rust Clients)
```bash
cargo run
```
This starts the WebSocket server and runs automated tests with three Rust clients.

### Web Mode (Browser Clients)
```bash
cargo run -- --web
```
This:
1. Starts the WebSocket server on port 8081
2. Serves a static web UI on http://localhost:8080
3. Allows testing with browser-based clients

## Project Structure
```
libws/
  ├── src/
  │   ├── lib.rs        # Core WebSocket server implementation
  │   └── ws_client.rs  # Rust client implementation
server/
  ├── src/
  │   ├── main.rs       # Server entry point
  │   └── client_tests.rs # Automated Rust client tests
  └── web/
      ├── index.html    # Web client UI
      └── tests.js      # JavaScript client implementation
```

## Dependencies
- Rust 2021 edition
- tokio for async runtime
- axum for WebSocket server
- serde_json for message serialization
- chrono for timestamp handling
