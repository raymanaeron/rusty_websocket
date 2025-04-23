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

    client1.subscribe(detect_event).await;
    client1.subscribe(connect_event).await;

    client2.subscribe(detect_event).await;
    client2.subscribe(registration_event).await;

    client3.subscribe(connect_event).await;
    client3.subscribe(registration_event).await;

    sleep(Duration::from_millis(200)).await;

    client1.publish(registration_event, "Registration complete").await;
    client2.publish(connect_event, "Network connected").await;
    client3.publish(detect_event, "Customer detected").await;

    sleep(Duration::from_secs(3)).await;
}
