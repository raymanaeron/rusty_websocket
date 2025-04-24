// Public module for WebSocket client functionality
pub mod ws_client;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{self, UnboundedSender};

// Type aliases for topic names and subscriber management
pub type Topic = String;
pub type Subscribers = Arc<Mutex<HashMap<Topic, Vec<UnboundedSender<String>>>>>;

/// Handles the WebSocket upgrade and initializes the connection.
pub async fn handle_socket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    subscribers: Subscribers,
) -> impl IntoResponse {
    println!("[handle_socket] WS connection from {}", addr);

    // Upgrade the connection and run the WebSocket handler
    ws.on_upgrade(move |socket| {
        async move {
            if let Err(e) = run_connection(socket, subscribers).await {
                eprintln!("[handle_socket] Client error: {:?}", e);
            }
        }
    })
}

/// Manages the WebSocket connection, handling messages, subscriptions, and publishing.
async fn run_connection(socket: WebSocket, subscribers: Subscribers) -> Result<(), String> {
    println!("[run_connection] Executing WebSocket connection handler...");

    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Track topics the client is subscribed to
    let my_topics = Arc::new(Mutex::new(Vec::<String>::new()));

    // Create a channel for sending messages to the client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let tx_clone = tx.clone();
    let subscribers_inner = subscribers.clone();
    let topics_inner = my_topics.clone();

    // Task for sending messages to the client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task for receiving messages from the client
    let receive_task = tokio::spawn(async move {
        let mut client_name = "<unknown>".to_string();
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    // Handle client name registration
                    if let Some(rest) = text.strip_prefix("register-name:") {
                        client_name = rest.trim().to_string();
                        println!("[register-name] => {}", client_name);

                    // Handle topic subscription
                    } else if let Some(rest) = text.strip_prefix("subscribe:") {
                        let topic = rest.trim().to_string();
                        println!("[subscribe] subscriber_name={}, topic={}", client_name, topic);

                        subscribers_inner
                            .lock()
                            .unwrap()
                            .entry(topic.clone())
                            .or_default()
                            .push(tx.clone());

                        topics_inner.lock().unwrap().push(topic);

                    // Handle topic unsubscription
                    } else if let Some(rest) = text.strip_prefix("unsubscribe:") {
                        let topic = rest.trim().to_string();
                        println!("[unsubscribe] {} unsubscribing from {}", client_name, topic);

                        let mut subs = subscribers_inner.lock().unwrap();
                        if let Some(vec) = subs.get_mut(&topic) {
                            vec.retain(|s| !same_channel(s, &tx));
                        }
                        topics_inner.lock().unwrap().retain(|t| t != &topic);

                    // Handle JSON message publishing
                    } else if let Some(rest) = text.strip_prefix("publish-json:") {
                        match serde_json::from_str::<Value>(rest) {
                            Ok(parsed) => {
                                let topic = parsed["topic"].as_str().unwrap_or("<none>").to_string();
                                let payload = parsed["payload"].as_str().unwrap_or("").to_string();
                                let publisher = parsed["publisher_name"].as_str().unwrap_or("<unknown>").to_string();
                                let timestamp = parsed["timestamp"].as_str().unwrap_or("").to_string();

                                println!(
                                    "[publish-json] publisher_name={}, topic={}, payload={}, timestamp={}",
                                    publisher, topic, payload, timestamp
                                );

                                let json_payload = json!({
                                    "publisher_name": publisher,
                                    "topic": topic,
                                    "payload": payload,
                                    "timestamp": timestamp
                                }).to_string();

                                let subs = subscribers_inner.lock().unwrap();
                                if let Some(sinks) = subs.get(&topic) {
                                    for s in sinks {
                                        if s.send(json_payload.clone()).is_err() {
                                            eprintln!("[publish-json] Failed to send to subscriber.");
                                        } else {
                                            println!("[publish-json] Sent to topic '{}'", topic);
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("[publish-json] Failed to parse JSON: {}", err);
                            }
                        }
                    }
                }
                Ok(_) => eprintln!("[run_connection] Received non-text message"),
                Err(e) => {
                    eprintln!("[run_connection] Error receiving: {:?}", e);
                    break;
                }
            }
        }
    });

    // Wait for both tasks to complete
    match tokio::try_join!(send_task, receive_task) {
        Ok(_) => println!("[run_connection] Connection closed cleanly."),
        Err(e) => {
            eprintln!("[run_connection] Task error: {:?}", e);
            return Err("WebSocket task crashed".into());
        }
    }

    // Cleanup subscriptions on client disconnect
    let mut subs = subscribers.lock().unwrap();
    for topic in my_topics.lock().unwrap().iter() {
        if let Some(vec) = subs.get_mut(topic) {
            vec.retain(|s| !same_channel(s, &tx_clone));
        }
    }

    println!("[run_connection] Cleanup complete.");
    Ok(())
}

/// Compares two channels to check if they are the same.
fn same_channel(a: &UnboundedSender<String>, b: &UnboundedSender<String>) -> bool {
    std::ptr::eq(a, b)
}
