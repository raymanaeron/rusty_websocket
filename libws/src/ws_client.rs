// src/ws_client.rs
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use futures_util::stream::{SplitSink, SplitStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::json;

type Callback = Box<dyn Fn(String) + Send + Sync>;

/// Represents a WebSocket client with per-topic message handlers.
pub struct WsClient {
    pub name: String, // The name of the client
    pub ws_channel: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, // WebSocket channel for sending messages
    on_message_handlers: Arc<Mutex<HashMap<String, Callback>>>, // Handlers for incoming messages by topic
    _async_task_handler: JoinHandle<()>, // Background task for receiving messages
}

impl WsClient {
    /// Connects to a WebSocket server and registers the client name.
    pub async fn connect(client_name: &str, ws_url: &str) -> tokio_tungstenite::tungstenite::Result<Self> {
        println!("[connect] client_name={}, ws_url={} -- executing", client_name, ws_url);

        // Establish the WebSocket connection
        let (stream, _) = connect_async(ws_url).await?;
        let (mut ws_channel, mut ws_receiver): (SplitSink<_, _>, SplitStream<_>) = stream.split();

        // Register the client name with the server
        let register_msg = format!("register-name:{}", client_name);
        ws_channel.send(Message::Text(register_msg)).await?;

        let name_clone = client_name.to_string();
        let handlers = Arc::new(Mutex::new(HashMap::<String, Callback>::new()));
        let handlers_clone = handlers.clone();

        // Spawn a task to handle incoming messages
        let task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Message::Text(txt) = msg {
                    match serde_json::from_str::<serde_json::Value>(&txt) {
                        Ok(parsed) => {
                            let topic = parsed.get("topic").and_then(|t| t.as_str()).unwrap_or("<unknown>");
                            let payload = parsed.get("payload").and_then(|m| m.as_str()).unwrap_or("<no message>");
                            let publisher = parsed.get("publisher_name").and_then(|p| p.as_str()).unwrap_or("<unknown>");
                            let timestamp = parsed.get("timestamp").and_then(|t| t.as_str()).unwrap_or("???");

                            println!(
                                "[on_message] {} <- topic={}, payload={}, publisher={}, timestamp={}",
                                name_clone, topic, payload, publisher, timestamp
                            );

                            // Invoke the callback for the topic if it exists
                            if let Some(callback) = handlers_clone.lock().unwrap().get(topic) {
                                callback(payload.to_string());
                            }
                        }
                        Err(_) => {
                            println!("[on_message] {} received malformed text: {}", name_clone, txt);
                        }
                    }
                }
            }
        });

        println!("[connect] client_name={} -- complete", client_name);

        Ok(Self {
            name: client_name.to_string(),
            ws_channel,
            on_message_handlers: handlers,
            _async_task_handler: task,
        })
    }

    /// Subscribes the client to a specific topic.
    pub async fn subscribe(&mut self, subscriber_name: &str, topic: &str, payload: &str) {
        println!("[subscribe] subscriber_name={}, topic={}, payload={}", subscriber_name, topic, payload);
        let cmd = format!("subscribe:{}", topic);
        if let Err(e) = self.ws_channel.send(Message::Text(cmd)).await {
            println!("[subscribe] Error: {:?}", e);
        }
    }

    /// Unsubscribes the client from a specific topic.
    pub async fn unsubscribe(&mut self, topic: &str) {
        println!("[unsubscribe] topic={}", topic);
        let cmd = format!("unsubscribe:{}", topic);
        if let Err(e) = self.ws_channel.send(Message::Text(cmd)).await {
            println!("[unsubscribe] Error: {:?}", e);
        }
    }

    /// Publishes a message to a specific topic.
    pub async fn publish(&mut self, publisher_name: &str, topic: &str, payload: &str, timestamp: &str) {
        println!("[publish] publisher_name={}, topic={}, payload={}, timestamp={}", publisher_name, topic, payload, timestamp);
        let msg = json!({
            "publisher_name": publisher_name,
            "topic": topic,
            "payload": payload,
            "timestamp": timestamp
        });
        let cmd = format!("publish-json:{}", msg.to_string());

        if let Err(e) = self.ws_channel.send(Message::Text(cmd)).await {
            println!("[publish] Error sending message: {:?}", e);
        }
    }

    /// Registers a callback to handle messages for a specific topic.
    pub fn on_message<F>(&mut self, topic: &str, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        println!("[on_message] registering handler for topic: {}", topic);
        self.on_message_handlers
            .lock()
            .unwrap()
            .insert(topic.to_string(), Box::new(callback));
    }
}
