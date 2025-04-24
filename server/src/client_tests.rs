// src/client_tests.rs
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

    // Connect three clients to the WebSocket server
    let mut client1 = WsClient::connect("Client1", url).await.unwrap();
    let mut client2 = WsClient::connect("Client2", url).await.unwrap();
    let mut client3 = WsClient::connect("Client3", url).await.unwrap();

    // Register message handlers for each client
    client1.on_message(detect_event, |msg| {
        println!("[Client1] => DetectCustomerEvent: {}", msg);
    });
    client1.on_message(connect_event, |msg| {
        println!("[Client1] => NetworkConnectedEvent: {}", msg);
    });

    client2.on_message(detect_event, |msg| {
        println!("[Client2] => DetectCustomerEvent: {}", msg);
    });
    client2.on_message(registration_event, |msg| {
        println!("[Client2] => RegistrationCompleteEvent: {}", msg);
    });

    client3.on_message(connect_event, |msg| {
        println!("[Client3] => NetworkConnectedEvent: {}", msg);
    });
    client3.on_message(registration_event, |msg| {
        println!("[Client3] => RegistrationCompleteEvent: {}", msg);
    });

    println!("[test] Subscribing clients to topics...");

    // Subscribe clients to specific topics
    client1.subscribe("Client1", detect_event, "no-payload").await;
    client1.subscribe("Client1", connect_event, "no-payload").await;

    client2.subscribe("Client2", detect_event, "no-payload").await;
    client2.subscribe("Client2", registration_event, "no-payload").await;

    client3.subscribe("Client3", connect_event, "no-payload").await;
    client3.subscribe("Client3", registration_event, "no-payload").await;

    // Allow some time for subscriptions to propagate
    sleep(Duration::from_millis(300)).await;

    println!("[test] Publishing messages...");

    // Publish messages to specific topics
    client1.publish("Client1", registration_event, "Registration complete", &timestamp).await;
    client2.publish("Client2", connect_event, "Network connected", &timestamp).await;
    client3.publish("Client3", detect_event, "Customer detected", &timestamp).await;

    // Wait to ensure all messages are processed
    sleep(Duration::from_secs(3)).await;

    println!("[test] Test complete.");
}
