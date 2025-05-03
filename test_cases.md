# Test Plan

## Setup

- Connect to a WebSocket server running locally at `ws://127.0.0.1:8081/ws`.
- Define three types of events that clients can listen for:
  - `DetectCustomerEvent`
  - `NetworkConnectedEvent`
  - `RegistrationCompleteEvent`
- Generate a current timestamp to include in messages for consistency and traceability.
- Define two distinct sessions to demonstrate session-based routing:
  - `session_a = "session-A"`
  - `session_b = "session-B"`

## Client Initialization

- Define two logical sessions: `session-A` and `session-B`.
- Clients connect with session:
  - `Client1` and `Client2` connect to `session-A`.
  - `Client3` and `Client4` connect to `session-B`.

## Message Handlers

- Each client registers callback functions to handle specific event types.
- When a message is received for a subscribed event, the client prints a log with its session ID and message content.

## Subscriptions

- Clients subscribe to specific events:
  - `Client1` → `DetectCustomerEvent`, `NetworkConnectedEvent`
  - `Client2` → `DetectCustomerEvent`, `RegistrationCompleteEvent`
  - `Client3` → `DetectCustomerEvent`, `NetworkConnectedEvent`
  - `Client4` → `RegistrationCompleteEvent`, `NetworkConnectedEvent`

## Publishing Messages

- Messages are published to different events, separated by session:
  - In `session-A`:
    - `Client1` publishes a `RegistrationCompleteEvent`
    - `Client2` publishes a `NetworkConnectedEvent`
  - In `session-B`:
    - `Client3` publishes a `DetectCustomerEvent`
    - `Client4` publishes a `RegistrationCompleteEvent`
- Each publish operation includes a timestamp and logs any failure to send.

## Execution Flow Control

- The test introduces brief delays between steps to allow subscriptions and message propagation.
- After publishing, it waits to ensure all messages are processed.

## Expected Behavior

- Messages are only delivered to clients that:
  - Subscribed to the event topic.
  - Belong to the same session as the message publisher.
- No cross-session message delivery should occur.

# Sequence Diagram

```mermaid
sequenceDiagram
    participant Client1
    participant Client2
    participant Client3
    participant Client4
    participant WebSocketServer

    Note over Client1,Client2: session-A
    Note over Client3,Client4: session-B

    Client1->>WebSocketServer: Connect with session-A
    Client2->>WebSocketServer: Connect with session-A
    Client3->>WebSocketServer: Connect with session-B
    Client4->>WebSocketServer: Connect with session-B

    Client1->>WebSocketServer: Subscribe to DetectCustomerEvent
    Client1->>WebSocketServer: Subscribe to NetworkConnectedEvent

    Client2->>WebSocketServer: Subscribe to DetectCustomerEvent
    Client2->>WebSocketServer: Subscribe to RegistrationCompleteEvent

    Client3->>WebSocketServer: Subscribe to DetectCustomerEvent
    Client3->>WebSocketServer: Subscribe to NetworkConnectedEvent

    Client4->>WebSocketServer: Subscribe to RegistrationCompleteEvent
    Client4->>WebSocketServer: Subscribe to NetworkConnectedEvent

    Note over Client1: Publish RegistrationCompleteEvent
    Client1->>WebSocketServer: Publish RegistrationCompleteEvent
    WebSocketServer-->>Client2: RegistrationCompleteEvent

    Note over Client2: Publish NetworkConnectedEvent
    Client2->>WebSocketServer: Publish NetworkConnectedEvent
    WebSocketServer-->>Client1: NetworkConnectedEvent

    Note over Client3: Publish DetectCustomerEvent
    Client3->>WebSocketServer: Publish DetectCustomerEvent
    WebSocketServer-->>Client3: DetectCustomerEvent

    Note over Client4: Publish RegistrationCompleteEvent
    Client4->>WebSocketServer: Publish RegistrationCompleteEvent
    WebSocketServer-->>Client4: RegistrationCompleteEvent

    Note over WebSocketServer: Messages isolated by session. No cross-session delivery.
```