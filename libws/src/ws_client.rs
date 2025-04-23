// src/ws_client.rs
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use futures_util::stream::{SplitSink, SplitStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Callback = Box<dyn Fn(String) + Send + Sync>;

/// Represents a WebSocket client with per-topic handlers.
pub struct WsClient {
    pub name: String,
    pub ws_channel: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    on_message_handlers: Arc<Mutex<HashMap<String, Callback>>>,
    _async_task_handler: JoinHandle<()>,
}

impl WsClient {
    pub async fn connect(name: &str, url: &str) -> tokio_tungstenite::tungstenite::Result<Self> {
        let (stream, _) = connect_async(url).await?;
        let (mut ws_channel, mut ws_receiver): (SplitSink<_, _>, SplitStream<_>) = stream.split();

        let register_msg = format!("register-name:{}", name);
        ws_channel.send(Message::Text(register_msg)).await?;

        let name_clone = name.to_string();
        let handlers = Arc::new(Mutex::new(HashMap::<String, Callback>::new()));
        let handlers_clone = handlers.clone();

        let task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Message::Text(txt) = msg {
                    if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(topic) = json_msg.get("topic").and_then(|t| t.as_str()) {
                            if let Some(payload) = json_msg.get("message").and_then(|m| m.as_str()) {
                                if let Some(callback) = handlers_clone.lock().unwrap().get(topic) {
                                    callback(payload.to_string());
                                } else {
                                    println!("[{name_clone}] Unhandled topic '{}': {}", topic, payload);
                                }
                            }
                        } else {
                            println!("[{name_clone}] Received unstructured message: {txt}");
                        }
                    } else {
                        println!("[{name_clone}] Received: {txt}");
                    }
                }
            }
        });

        Ok(Self {
            name: name.to_string(),
            ws_channel,
            on_message_handlers: handlers,
            _async_task_handler: task,
        })
    }

    pub async fn subscribe(&mut self, topic: &str) {
        let cmd = format!("subscribe:{}", topic);
        let _ = self.ws_channel.send(Message::Text(cmd)).await;
    }

    pub async fn unsubscribe(&mut self, topic: &str) {
        let cmd = format!("unsubscribe:{}", topic);
        let _ = self.ws_channel.send(Message::Text(cmd)).await;
    }

    pub async fn publish(&mut self, topic: &str, payload: &str) {
        let cmd = format!("publish:{}:{}", topic, payload);
        let _ = self.ws_channel.send(Message::Text(cmd)).await;
    }

    pub fn on_message<F>(&mut self, topic: &str, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_message_handlers
            .lock()
            .unwrap()
            .insert(topic.to_string(), Box::new(callback));
    }
}
