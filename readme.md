# TODO: THIS IS A WORK IN PROGRESS
- The encryption/decryption working ok but not hooked up with the web socket server. This is a tough one and I need to pick it up some other time
- The web socket server has session based pub/sub implmentation which are hooked up and working properly
- JWT authentication has been added for secure connections

# WebSocket-based Pub/Sub Framework for Distributed Applications

## Problem Statement
Modern applications often consist of multiple distributed components that need to communicate in real-time. Traditional HTTP-based communication can be cumbersome and doesn't support real-time updates efficiently. Additionally, managing state and coordination between multiple clients (web, desktop, CLI) becomes complex without a centralized messaging system.

When multiple users or devices connect to the same messaging system, there's a critical need for session isolation. Without proper session boundaries, messages intended for one user's session might be delivered to another user's session, creating privacy concerns and data leakage. This is especially problematic in multi-tenant applications where different user sessions must remain strictly isolated.

Furthermore, modern real-time communication systems require robust authentication and authorization mechanisms. Unauthenticated WebSocket connections can lead to unauthorized access, data breaches, and potential impersonation attacks. Standard cookie-based authentication is insufficient for WebSockets, especially across different domains or in non-browser environments. A token-based authentication system like JWT is necessary to verify user identity, maintain secure sessions, and provide fine-grained access control while supporting both browser and non-browser clients.

## Solution
This framework provides a lightweight pub/sub (publish-subscribe) messaging system using WebSocket technology. It includes:
- A high-performance Rust WebSocket server using axum
- A Rust client library with async support and error handling
- A JavaScript client implementation for web browsers
- JSON-based message protocol for cross-platform compatibility
- Session-based message routing to ensure privacy and data isolation between users
- JWT-based authentication for secure connections

## Benefits
- Real-time bidirectional communication
- Language-agnostic messaging (Rust, JavaScript)
- Topic-based message routing
- Connection state management and error handling
- Simple API for both Rust and JavaScript clients
- Session-scoped messaging to prevent cross-session data leakage
- Proper isolation between different user sessions or application instances
- Secure authentication with JWT tokens
- Integration of authentication with the session system

## Architecture

```
[Rust Client] ←→ [WebSocket Server] ←→ [Browser Client]
     ↑               ↑                        ↑
     |               |                        |
   connect()     JSON Message              connect()
   subscribe()    Protocol              subscribe()
   publish()                            publish()
   auth_token()                         auth_token()
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

// Or connect with JWT authentication
let mut client = WsClient::connect_with_auth(
    "Client1",
    "ws://127.0.0.1:8081/ws",
    "http://127.0.0.1:8081/auth/token",
    "username",
    "password",
    Some("user-session-123")
).await?;
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

### JWT Token Management
```rust
// Check if client is authenticated
if client.is_authenticated() {
    println!("Client is authenticated via JWT");
}

// Get the current token if needed
if let Some(token) = client.get_token() {
    println!("Current JWT token: {}", token);
}

// Refresh token if needed
if let Ok(refreshed) = client.refresh_token_if_needed().await {
    if refreshed {
        println!("Token was refreshed");
    } else {
        println!("Token is still valid");
    }
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

### Authenticated Connection
```javascript
import { jwtManager, createAuthenticatedWebSocket } from './jwt_utils.js';

// Authenticate and create WebSocket connection
const ws = await createAuthenticatedWebSocket(
    "ws://localhost:8081/ws", 
    "http://localhost:8081/auth/token", 
    "username", 
    "password", 
    "user-session-456"
);

// Subscribe to topics
ws.send(`subscribe:AuthenticatedTestTopic|user-session-456`);

// Set up message handler
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log(`Received message: ${data.payload}`);
};
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

### JWT Token Management
```javascript
// Check if authenticated
if (jwtManager.isAuthenticated()) {
    console.log("Client is authenticated");
}

// Get current token
const token = jwtManager.getToken();
if (token) {
    console.log("Have valid token");
}

// Clear token (e.g., for logout)
jwtManager.clearToken();
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
  │   ├── ws_client.rs  # Rust client implementation
  │   ├── jwt_utils.rs  # JWT utilities for token handling
  │   └── jwt_api_route.rs # JWT authentication API
server/
  ├── src/
  │   ├── main.rs       # Server entry point
  │   └── client_tests.rs # Automated Rust client tests
  └── web/
      ├── index.html    # Web client UI
      ├── tests.js      # JavaScript client implementation
      ├── jwt_utils.js  # JWT utilities for JavaScript
      └── jwt_tests.html # JWT authentication test page
```

## Dependencies
- Rust 2021 edition
- tokio for async runtime
- axum for WebSocket server
- serde_json for message serialization
- chrono for timestamp handling
- jsonwebtoken for JWT authentication
- reqwest for HTTP client functionality

## JWT Authentication Configuration

The JWT authentication system can be configured using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| JWT_SECRET_KEY | Secret key used to sign JWTs | "rusty_websocket_jwt_secret_key_32b" |
| JWT_EXPIRATION_SECONDS | Token expiration time in seconds | 3600 (1 hour) |

### JWT Authentication Flow

1. Client requests a token via the `/auth/token` endpoint, providing username, password, and optional session ID
2. Server validates credentials and issues a JWT token containing user identity and session ID
3. Client includes this token in WebSocket connection URL as a query parameter
4. Server validates the token and establishes an authenticated WebSocket connection
5. Session ID from the token is used for message routing

### JWT Token Structure

```json
{
  "sub": "username",     // Subject (user identifier)
  "sid": "session-123",  // Session ID (optional)
  "iat": 1714597440,     // Issued at time
  "exp": 1714601040      // Expiration time
}
```

### Example: Using JWT with curl

```bash
# Get a JWT token
curl -X POST http://localhost:8081/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password","session_id":"my-session"}'

# Response will be like:
# {"token":"eyJhbGciOiJIUzI1NiJ9...","expires_in":3600}
```
````markdown
