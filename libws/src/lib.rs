pub mod ws_client;

use axum::{
    extract::ws::{ Message, WebSocket, WebSocketUpgrade },
    extract::ConnectInfo,
    response::IntoResponse,
};
use futures_util::{ SinkExt, StreamExt };
use std::{ collections::HashMap, net::SocketAddr, sync::{ Arc, Mutex } };
use tokio::sync::mpsc::{ self, UnboundedSender };

pub type Topic = String;
pub type Subscribers = Arc<Mutex<HashMap<Topic, Vec<UnboundedSender<String>>>>>;

pub async fn handle_socket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    subscribers: Subscribers
) -> impl IntoResponse {
    println!("[server] WS connection from {}", addr);

    ws.on_upgrade(move |socket| {
        // Await the connection directly, do not spawn!
        async move {
            if let Err(e) = run_connection(socket, subscribers).await {
                eprintln!("[server] Client error: {:?}", e);
            }
        }
    })
}

async fn run_connection(socket: WebSocket, subscribers: Subscribers) -> Result<(), String> {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let my_topics = Arc::new(Mutex::new(Vec::<String>::new()));
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let tx_clone = tx.clone();
    let subscribers_inner = subscribers.clone();
    let topics_inner = my_topics.clone();

    // Spawn task for sending messages to client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn task for receiving messages from client
    let receive_task = tokio::spawn(async move {
        let mut client_name: String = "<unknown>".to_string();
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    let content = text.clone(); // still needed
    
                    if let Some(rest) = content.strip_prefix("register-name:") {
                        client_name = rest.trim().to_string();
                        println!("[server] Registered client name: [{}]", client_name);
                    } else if let Some(rest) = content.strip_prefix("subscribe:") {
                        let topic = rest.trim().to_string();
                        println!("[server] [{}] subscribed to topic: {}", client_name, topic);
    
                        subscribers_inner
                            .lock()
                            .unwrap()
                            .entry(topic.clone())
                            .or_default()
                            .push(tx.clone());
    
                        topics_inner.lock().unwrap().push(topic);
                    } else if let Some(rest) = content.strip_prefix("unsubscribe:") {
                        let topic = rest.trim().to_string();
                        println!("[server] [{}] unsubscribed from topic: {}", client_name, topic);
    
                        let mut subs = subscribers_inner.lock().unwrap();
                        if let Some(vec) = subs.get_mut(&topic) {
                            vec.retain(|s| !same_channel(s, &tx));
                        }
    
                        topics_inner.lock().unwrap().retain(|t| t != &topic);
                    } else if let Some(rest) = content.strip_prefix("publish:") {
                        if let Some((topic, payload)) = rest.trim().split_once(":") {
                            let topic = topic.trim().to_string();
                            let message = payload.trim().to_string();
    
                            println!("[server] [{}] publishing to {}: {}", client_name, topic, message);
    
                            let subs = subscribers_inner.lock().unwrap();
                            if let Some(sinks) = subs.get(&topic) {
                                for s in sinks {
                                    if s.send(message.clone()).is_err() {
                                        eprintln!("[server] [{}] failed to send to subscriber", client_name);
                                    } else {
                                        println!("[server] [{}] -> sent to topic [{}]: {}", client_name, topic, message);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(_) => {
                    eprintln!("[server] [{}] Received non-text message. Ignoring.", client_name);
                }
                Err(e) => {
                    eprintln!("[server] [{}] receive_task socket error: {:?}", client_name, e);
                    break;
                }
            }
        }
    });   

    // Wait for both tasks to complete
    match tokio::try_join!(send_task, receive_task) {
        Ok((_, _)) => {}
        Err(e) => {
            eprintln!("[server] Task join error: {:?}", e);
            return Err("WebSocket task crashed".into());
        }
    }

    // Cleanup on disconnect
    let mut subs = subscribers.lock().unwrap();
    for topic in my_topics.lock().unwrap().iter() {
        if let Some(vec) = subs.get_mut(topic) {
            vec.retain(|s| !same_channel(s, &tx_clone));
        }
    }

    println!("[server] Client disconnected and cleaned up.");
    Ok(())
}

fn same_channel(a: &UnboundedSender<String>, b: &UnboundedSender<String>) -> bool {
    std::ptr::eq(a, b)
}
