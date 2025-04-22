// src/client_tests.rs
use libws::ws_client::WsClient;
use tokio::time::{sleep, Duration};

/// Runs a series of client tests to simulate WebSocket interactions.
pub async fn run_client_tests() {
    let url = "ws://127.0.0.1:8081/ws"; // WebSocket server URL

    // Define event topics
    let detect_event = "DetectCustomerEvent";
    let connect_event = "NetworkConnectedEvent";
    let registration_event = "RegistrationCompleteEvent";

    // Connect three clients to the WebSocket server
    let mut client1 = WsClient::connect("Client1", url).await.unwrap();
    let mut client2 = WsClient::connect("Client2", url).await.unwrap();
    let mut client3 = WsClient::connect("Client3", url).await.unwrap();

    // Subscribe clients to specific topics
    client1.subscribe(detect_event).await;
    client1.subscribe(connect_event).await;

    client2.subscribe(detect_event).await;
    client2.subscribe(registration_event).await;

    client3.subscribe(connect_event).await;
    client3.subscribe(registration_event).await;

    // Allow some time for subscriptions to propagate
    sleep(Duration::from_millis(200)).await;

    // Publish messages to specific topics
    println!("[Client1] Publishing to RegistrationCompleteEvent...");
    client1.publish(registration_event, "Registration complete").await;

    println!("[Client2] Publishing to NetworkConnectedEvent...");
    client2.publish(connect_event, "Network connected").await;

    println!("[Client3] Publishing to DetectCustomerEvent...");
    client3.publish(detect_event, "Customer detected").await;

    // Wait to ensure all messages are processed
    sleep(Duration::from_secs(3)).await;
}
