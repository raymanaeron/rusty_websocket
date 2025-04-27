// src/ws_tests.rs
use libws::ws_client::WsClient;
use tokio::time::{sleep, Duration};
use chrono::Utc;

/// Runs a series of client tests to simulate WebSocket interactions.
pub async fn run_client_tests() {
    let url = "ws://127.0.0.1:8081/ws"; // WebSocket server URL

    // Define event topics
    let detect_event = "DetectCustomerEvent";
    let connect_event = "NetworkConnectedEvent";
    let registration_event = "RegistrationCompleteEvent";

    // Generate a timestamp for the test
    let timestamp = Utc::now().to_rfc3339();

    println!("[test] Connecting clients...");

    // Define two distinct sessions to demonstrate session-based routing
    let session_a = "session-A";
    let session_b = "session-B";

    // Connect four clients to the WebSocket server, with different sessions
    let mut client1 = WsClient::connect_with_session("Client1", session_a, url).await.unwrap();
    let mut client2 = WsClient::connect_with_session("Client2", session_a, url).await.unwrap();
    let mut client3 = WsClient::connect_with_session("Client3", session_b, url).await.unwrap();
    let mut client4 = WsClient::connect_with_session("Client4", session_b, url).await.unwrap();

    // Register message handlers for each client
    // Added 'move' keyword to all closures to take ownership of captured variables
    client1.on_message(detect_event, move |msg| {
        println!("[Client1:{}] => DetectCustomerEvent: {}", session_a, msg);
    });
    client1.on_message(connect_event, move |msg| {
        println!("[Client1:{}] => NetworkConnectedEvent: {}", session_a, msg);
    });

    client2.on_message(detect_event, move |msg| {
        println!("[Client2:{}] => DetectCustomerEvent: {}", session_a, msg);
    });
    client2.on_message(registration_event, move |msg| {
        println!("[Client2:{}] => RegistrationCompleteEvent: {}", session_a, msg);
    });

    client3.on_message(detect_event, move |msg| {
        println!("[Client3:{}] => DetectCustomerEvent: {}", session_b, msg);
    });
    client3.on_message(connect_event, move |msg| {
        println!("[Client3:{}] => NetworkConnectedEvent: {}", session_b, msg);
    });
    
    client4.on_message(registration_event, move |msg| {
        println!("[Client4:{}] => RegistrationCompleteEvent: {}", session_b, msg);
    });
    client4.on_message(connect_event, move |msg| {
        println!("[Client4:{}] => NetworkConnectedEvent: {}", session_b, msg);
    });

    println!("[test] Subscribing clients to topics...");

    // Subscribe clients to specific topics
    client1.subscribe("Client1", detect_event, "no-payload").await;
    client1.subscribe("Client1", connect_event, "no-payload").await;

    client2.subscribe("Client2", detect_event, "no-payload").await;
    client2.subscribe("Client2", registration_event, "no-payload").await;

    client3.subscribe("Client3", detect_event, "no-payload").await;
    client3.subscribe("Client3", connect_event, "no-payload").await;
    
    client4.subscribe("Client4", registration_event, "no-payload").await;
    client4.subscribe("Client4", connect_event, "no-payload").await;

    // Allow some time for subscriptions to propagate
    sleep(Duration::from_millis(300)).await;

    println!("[test] Publishing messages...");

    // Publish messages to specific topics in session A
    if let Err(e) = client1.publish("Client1", registration_event, "Registration complete from session A", &timestamp).await {
        println!("[Client1] Publish failed: {}", e);
    }
    if let Err(e) = client2.publish("Client2", connect_event, "Network connected in session A", &timestamp).await {
        println!("[Client2] Publish failed: {}", e);
    }
    
    // Wait a bit before sending the next batch
    sleep(Duration::from_millis(500)).await;
    
    // Publish messages to specific topics in session B
    if let Err(e) = client3.publish("Client3", detect_event, "Customer detected in session B", &timestamp).await {
        println!("[Client3] Publish failed: {}", e);
    }
    if let Err(e) = client4.publish("Client4", registration_event, "Registration complete from session B", &timestamp).await {
        println!("[Client4] Publish failed: {}", e);
    }
    
    // Wait to ensure all messages are processed
    sleep(Duration::from_secs(3)).await;

    println!("[test] Test complete. Messages were only delivered within their respective sessions.");
}