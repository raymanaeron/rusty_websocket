// src/client_tests.rs
use libws::ws_client::WsClient;
use tokio::time::{sleep, Duration};

pub async fn run_client_tests() {
    let url = "ws://127.0.0.1:8081/ws";

    let detect_event = "DetectCustomerEvent";
    let connect_event = "NetworkConnectedEvent";
    let registration_event = "RegistrationCompleteEvent";

    let mut client1 = WsClient::connect("Client1", url).await.unwrap();
    let mut client2 = WsClient::connect("Client2", url).await.unwrap();
    let mut client3 = WsClient::connect("Client3", url).await.unwrap();

    client1.subscribe(detect_event).await;
    client1.subscribe(connect_event).await;

    client2.subscribe(detect_event).await;
    client2.subscribe(registration_event).await;

    client3.subscribe(connect_event).await;
    client3.subscribe(registration_event).await;

    sleep(Duration::from_millis(200)).await;

    println!("[Client1] Publishing to RegistrationCompleteEvent...");
    client1.publish(registration_event, "Registration complete").await;

    println!("[Client2] Publishing to NetworkConnectedEvent...");
    client2.publish(connect_event, "Network connected").await;

    println!("[Client3] Publishing to DetectCustomerEvent...");
    client3.publish(detect_event, "Customer detected").await;

    sleep(Duration::from_secs(3)).await;
}
