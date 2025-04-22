// src/ws_client.rs
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite;
use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use futures_util::stream::{SplitSink, SplitStream};

pub struct WsClient {
    pub name: String,
    pub ws_channel: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    _async_task_handler: JoinHandle<()>,
}

impl WsClient {
    pub async fn connect(name: &str, url: &str) -> tungstenite::Result<Self> {
        let (stream, _) = connect_async(url).await?;
        let (mut ws_channel, mut ws_receiver): (SplitSink<_, _>, SplitStream<_>) = stream.split();

        // Register the client name on the server side
        let register_msg = format!("register-name:{}", name);
        ws_channel.send(Message::Text(register_msg)).await?;

        let client_name = name.to_string();
        let name_clone = client_name.clone();

        let task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Message::Text(txt) = msg {
                    println!("[{name_clone}] Received: {txt}");
                }
            }
        });

        Ok(Self {
            name: client_name,
            ws_channel,
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
}
