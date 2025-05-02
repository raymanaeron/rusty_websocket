// Public module for WebSocket client functionality
pub mod ws_client;
pub mod enc_utils;
pub mod enc_api_route;
pub mod jwt_utils;
pub mod jwt_api_route;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{ConnectInfo, Query},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    env,
};
use serde::Deserialize;
use tokio::sync::mpsc::{self, UnboundedSender};
use crate::jwt_utils::{validate_token, Claims};

// Type aliases for topic names and subscriber management
pub type Topic = String;
pub type SessionId = String;
// New type: Map of topics to a map of session IDs to subscribers
pub type Subscribers = Arc<Mutex<HashMap<Topic, HashMap<SessionId, Vec<UnboundedSender<String>>>>>>;

// Query parameters struct for WebSocket connections
#[derive(Deserialize, Debug)]
pub struct WebSocketParams {
    token: Option<String>,
}

/// Handles the WebSocket upgrade and initializes the connection.
pub async fn handle_socket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    params: Option<Query<WebSocketParams>>, // Add query parameters to extract token
    subscribers: Subscribers,
) -> impl IntoResponse {
    println!("[handle_socket] WS connection from {}", addr);
    
    // Extract token from query parameters if present
    let token = params.as_ref().and_then(|p| p.token.clone());

    // Check if we have a token (for authenticated connections)
    let user_info = if let Some(token_str) = token {
        // Get JWT secret from environment variable or use default
        let secret = env::var("JWT_SECRET_KEY")
            .map(|s| s.into_bytes())
            .unwrap_or_else(|_| b"rusty_websocket_jwt_secret_key_32b".to_vec());
        
        // Try to validate the token
        match validate_token(&token_str, &secret) {
            Ok(claims) => {
                println!("[handle_socket] Validated JWT for user: {}", claims.sub);
                Some(claims)
            },
            Err(e) => {
                println!("[handle_socket] Invalid JWT token: {}", e);
                None
            }
        }
    } else {
        println!("[handle_socket] No JWT token provided");
        None
    };

    // Upgrade the connection and run the WebSocket handler
    ws.on_upgrade(move |socket| {
        async move {
            if let Err(e) = run_connection(socket, subscribers, user_info).await {
                eprintln!("[handle_socket] Client error: {:?}", e);
            }
        }
    })
}

/// Manages the WebSocket connection, handling messages, subscriptions, and publishing.
async fn run_connection(
    socket: WebSocket, 
    subscribers: Subscribers,
    user_info: Option<Claims>
) -> Result<(), String> {
    println!("[run_connection] Executing WebSocket connection handler...");
    
    // Extract user ID and associated session ID from token claims
    let (user_id, token_session_id) = if let Some(claims) = &user_info {
        println!("[run_connection] JWT claims: user_id={}, session_id={:?}", 
            claims.sub, claims.sid);
        (
            Some(claims.sub.clone()),
            claims.sid.clone()
        )
    } else {
        println!("[run_connection] No JWT claims available");
        (None, None)
    };

    if let Some(id) = &user_id {
        println!("[run_connection] Authenticated connection for user: {}", id);
    } else {
        println!("[run_connection] Anonymous connection");
    }

    // Split the WebSocket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Track topics the client is subscribed to
    let my_subscriptions = Arc::new(Mutex::new(Vec::<(String, String)>::new())); // Now stores (topic, sessionId) pairs

    // Create a channel for sending messages to the client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let tx_clone = tx.clone();
    let subscribers_inner = subscribers.clone();
    let subscriptions_inner = my_subscriptions.clone();

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
        // Fix 1: Use clone to avoid moving user_id
        let user_id_for_name = user_id.clone();
        let mut client_name = user_id_for_name.unwrap_or_else(|| "<unknown>".to_string());
        
        // Fix 2: Use clone to avoid moving token_session_id
        let token_session_id_for_session = token_session_id.clone();
        let mut session_id = token_session_id_for_session.unwrap_or_else(|| "default".to_string());
        
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    // Handle client name registration
                    if let Some(rest) = text.strip_prefix("register-name:") {
                        // If authenticated, don't allow changing the client name
                        if user_id.is_none() {
                            client_name = rest.trim().to_string();
                            println!("[register-name] => {}", client_name);
                        } else {
                            println!("[register-name] Ignoring name registration for authenticated user");
                        }

                    // Handle session ID registration
                    } else if let Some(rest) = text.strip_prefix("register-session:") {
                        // If token has session ID, don't allow changing it
                        if token_session_id.is_none() {
                            session_id = rest.trim().to_string();
                            println!("[register-session] {} => {}", client_name, session_id);
                        } else {
                            println!("[register-session] Ignoring session registration, using token session");
                        }

                    // Handle topic subscription
                    } else if let Some(rest) = text.strip_prefix("subscribe:") {
                        let parts: Vec<&str> = rest.trim().split("|").collect();
                        let topic = parts[0].to_string();
                        
                        // KEY FIX: Use provided session ID, or session ID from token, or default session ID
                        let sub_session_id = if parts.len() > 1 { 
                            parts[1].to_string() 
                        } else if token_session_id.is_some() {
                            // Use token session ID if available - this is the critical fix
                            session_id.clone()
                        } else { 
                            session_id.clone() 
                        };
                        
                        println!("[subscribe] subscriber_name={}, topic={}, session={}", 
                            client_name, topic, sub_session_id);
                        println!("[subscribe] Using session ID from token: {}", session_id);

                        let mut subs = subscribers_inner.lock().unwrap();
                        subs.entry(topic.clone())
                            .or_insert_with(HashMap::new)
                            .entry(sub_session_id.clone())
                            .or_insert_with(Vec::new)
                            .push(tx.clone());

                        println!("[subscribe] Subscription added for topic={}, session={}", 
                            topic, sub_session_id);
                        subscriptions_inner.lock().unwrap().push((topic, sub_session_id));

                    // Handle topic unsubscription
                    } else if let Some(rest) = text.strip_prefix("unsubscribe:") {
                        let parts: Vec<&str> = rest.trim().split("|").collect();
                        let topic = parts[0].to_string();
                        // Use provided session ID or fallback to the client's session ID
                        let unsub_session_id = if parts.len() > 1 { parts[1].to_string() } else { session_id.clone() };
                        
                        println!("[unsubscribe] {} unsubscribing from {} in session {}", client_name, topic, unsub_session_id);

                        let mut subs = subscribers_inner.lock().unwrap();
                        if let Some(session_map) = subs.get_mut(&topic) {
                            if let Some(vec) = session_map.get_mut(&unsub_session_id) {
                                vec.retain(|s| !same_channel(s, &tx));
                                if vec.is_empty() {
                                    session_map.remove(&unsub_session_id);
                                }
                            }
                        }
                        
                        subscriptions_inner.lock().unwrap().retain(|t| !(t.0 == topic && t.1 == unsub_session_id));
                    
                    // Handle JSON message publishing
                    } else if let Some(rest) = text.strip_prefix("publish-json:") {
                        match serde_json::from_str::<Value>(rest) {
                            Ok(parsed) => {
                                let topic = parsed["topic"].as_str().unwrap_or("<none>").to_string();
                                let payload = parsed["payload"].as_str().unwrap_or("").to_string();
                                let publisher = parsed["publisher_name"].as_str().unwrap_or("<unknown>").to_string();
                                let timestamp = parsed["timestamp"].as_str().unwrap_or("").to_string();
                                // Extract session ID from JSON or use default
                                let pub_session_id = parsed["session_id"].as_str().unwrap_or(&session_id).to_string();

                                println!(
                                    "[publish-json] publisher_name={}, topic={}, payload={}, timestamp={}, session={}",
                                    publisher, topic, payload, timestamp, pub_session_id
                                );

                                let json_payload = json!({
                                    "publisher_name": publisher,
                                    "topic": topic,
                                    "payload": payload,
                                    "timestamp": timestamp,
                                    "session_id": pub_session_id
                                }).to_string();

                                let subs = subscribers_inner.lock().unwrap();
                                if let Some(session_map) = subs.get(&topic) {
                                    // Only send to subscribers of the same session
                                    println!("[publish-json] Session map has {} entries", session_map.len());
                                    for (sess_id, _) in session_map.iter() {
                                        println!("[publish-json] Available session: {}", sess_id);
                                    }
                                    
                                    if let Some(sinks) = session_map.get(&pub_session_id) {
                                        println!("[publish-json] Found {} subscribers for session {}", sinks.len(), pub_session_id);
                                        for s in sinks {
                                            if s.send(json_payload.clone()).is_err() {
                                                eprintln!("[publish-json] Failed to send to subscriber.");
                                            } else {
                                                println!("[publish-json] Sent to topic '{}' in session '{}'", topic, pub_session_id);
                                            }
                                        }
                                    } else {
                                        println!("[publish-json] No subscribers found for session '{}'", pub_session_id);
                                    }
                                } else {
                                    println!("[publish-json] No session map found for topic '{}'", topic);
                                }
                            }
                            Err(err) => {
                                eprintln!("[publish-json] Failed to parse JSON: {}", err);
                                println!("[publish-json] Raw JSON: {}", rest);
                            }
                        }
                    } else if text == "ping" {
                        println!("[ping] Received ping message");
                        // Send a pong response
                        if tx.send("pong".to_string()).is_err() {
                            eprintln!("[ping] Failed to send pong response");
                        } else {
                            println!("[ping] Sent pong response");
                        }
                    } else {
                        println!("[unknown] Received unknown message: {}", text);
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
    for (topic, session_id) in my_subscriptions.lock().unwrap().iter() {
        if let Some(session_map) = subs.get_mut(topic) {
            if let Some(vec) = session_map.get_mut(session_id) {
                vec.retain(|s| !same_channel(s, &tx_clone));
                if vec.is_empty() {
                    session_map.remove(session_id);
                }
            }
            if session_map.is_empty() {
                subs.remove(topic);
            }
        }
    }

    println!("[run_connection] Cleanup complete.");
    Ok(())
}

/// Compares two channels to check if they are the same.
fn same_channel(a: &UnboundedSender<String>, b: &UnboundedSender<String>) -> bool {
    std::ptr::eq(a, b)
}
