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
use std::time::{Duration, Instant};
use std::error::Error;

// Add JWT-related imports
use reqwest;
use serde::Deserialize;
use url::Url;

type Callback = Box<dyn Fn(String) + Send + Sync>;

/// JWT Auth Response from the server
#[derive(Debug, Deserialize)]
struct JwtAuthResponse {
    token: String,
    expires_in: u64,
}

/// Represents a WebSocket client with per-topic message handlers.
pub struct WsClient {
    pub name: String, // The name of the client
    pub session_id: String, // The session ID for this client
    pub ws_channel: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, // WebSocket channel for sending messages
    on_message_handlers: Arc<Mutex<HashMap<String, Callback>>>, // Handlers for incoming messages by topic
    _async_task_handler: JoinHandle<()>, // Background task for receiving messages
    is_connected: Arc<Mutex<bool>>, // Tracks the connection state
    // New fields for JWT authentication
    auth_token: Arc<Mutex<Option<String>>>, // JWT token if authenticated
    token_expiry: Arc<Mutex<Option<Instant>>>, // When the token expires
    auth_url: Option<String>, // URL for token refresh
}

impl WsClient {
    /// Connects to a WebSocket server and registers the client name.
    pub async fn connect(client_name: &str, ws_url: &str) -> tokio_tungstenite::tungstenite::Result<Self> {
        // Use a default session ID derived from client name
        let session_id = format!("session-{}", client_name);
        Self::connect_with_session(client_name, session_id.as_str(), ws_url).await
    }

    /// Connects to a WebSocket server with a specific session ID.
    pub async fn connect_with_session(
        client_name: &str, 
        session_id: &str, 
        ws_url: &str
    ) -> tokio_tungstenite::tungstenite::Result<Self> {
        println!("[connect] client_name={}, session_id={}, ws_url={} -- executing", 
            client_name, session_id, ws_url);

        // Establish the WebSocket connection
        let (stream, _) = connect_async(ws_url).await?;
        let (mut ws_channel, mut ws_receiver): (SplitSink<_, _>, SplitStream<_>) = stream.split();

        // Register the client name with the server
        let register_msg = format!("register-name:{}", client_name);
        ws_channel.send(Message::Text(register_msg)).await?;
        
        // Register the session ID with the server
        let register_session = format!("register-session:{}", session_id);
        ws_channel.send(Message::Text(register_session)).await?;

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
                            let msg_session = parsed.get("session_id").and_then(|s| s.as_str()).unwrap_or("<unknown>");

                            println!(
                                "[on_message] {} <- topic={}, payload={}, publisher={}, timestamp={}, session={}",
                                name_clone, topic, payload, publisher, timestamp, msg_session
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

        println!("[connect] client_name={}, session_id={} -- complete", client_name, session_id);

        Ok(Self {
            name: client_name.to_string(),
            session_id: session_id.to_string(),
            ws_channel,
            on_message_handlers: handlers,
            _async_task_handler: task,
            is_connected: Arc::new(Mutex::new(true)),
            auth_token: Arc::new(Mutex::new(None)),
            token_expiry: Arc::new(Mutex::new(None)),
            auth_url: None,
        })
    }

    /// Connects to a WebSocket server with JWT authentication
    pub async fn connect_with_auth(
        client_name: &str,
        ws_url: &str,
        auth_url: &str,
        username: &str,
        password: &str,
        session_id: Option<&str>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        println!("[connect_with_auth] Getting JWT token for {}...", username);
        
        // Get JWT token from auth endpoint
        let token_result = Self::get_auth_token(auth_url, username, password, session_id).await?;
        let token = token_result.token;
        
        // Calculate token expiry time
        let expires_at = Instant::now() + Duration::from_secs(token_result.expires_in);
        
        println!("[connect_with_auth] JWT token obtained, expires in {} seconds", token_result.expires_in);
        
        // Modify WebSocket URL to include token as a query parameter
        let mut ws_url_with_token = Url::parse(ws_url)?;
        ws_url_with_token.query_pairs_mut().append_pair("token", &token);
        
        // Connect to WebSocket with the token
        let client = Self::connect(client_name, ws_url_with_token.as_str()).await?;
        
        // Update authentication fields
        {
            let mut auth_token = client.auth_token.lock().unwrap();
            *auth_token = Some(token);
            
            let mut token_expiry = client.token_expiry.lock().unwrap();
            *token_expiry = Some(expires_at);
        }
        
        // Store auth URL for potential token refresh
        let mut client = client;
        client.auth_url = Some(auth_url.to_string());
        
        println!("[connect_with_auth] Authenticated connection established for {}", username);
        Ok(client)
    }

    /// Gets a JWT auth token from the server
    async fn get_auth_token(
        auth_url: &str, 
        username: &str, 
        password: &str,
        session_id: Option<&str>,
    ) -> Result<JwtAuthResponse, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();
        
        // Prepare the authentication request
        let mut auth_request = serde_json::json!({
            "username": username,
            "password": password
        });
        
        // Add session ID if provided
        if let Some(sid) = session_id {
            auth_request["session_id"] = serde_json::Value::String(sid.to_string());
        }
        
        // Make the POST request to get the token
        let response = client
            .post(auth_url)
            .json(&auth_request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(format!("Authentication failed: HTTP {}", response.status()).into());
        }
        
        // Parse the JWT response
        let token_response = response.json::<JwtAuthResponse>().await?;
        Ok(token_response)
    }

    /// Refreshes the JWT token if needed
    pub async fn refresh_token_if_needed(&mut self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let needs_refresh = {
            let expiry = self.token_expiry.lock().unwrap();
            match *expiry {
                Some(expires_at) => {
                    // Refresh if token will expire in the next 5 minutes
                    let five_min = Duration::from_secs(300);
                    expires_at.checked_duration_since(Instant::now())
                        .map_or(true, |remaining| remaining < five_min)
                },
                None => false, // No token, so no need to refresh
            }
        };
        
        // If token needs refreshing and we have an auth URL
        if needs_refresh {
            if let Some(auth_url) = &self.auth_url {
                // We need to re-authenticate - this would typically use a refresh token
                // but for this example we'll assume we have the username/password stored
                // In a real app, you'd use a more secure token refresh mechanism
                println!("[refresh_token] Token expiring soon, refreshing...");
                
                // This is placeholder code - in a real app you'd implement a proper token refresh
                // This just demonstrates the concept of refreshing a token
                let token_result = Self::get_auth_token(
                    auth_url, 
                    &self.name, 
                    "placeholder_password", 
                    Some(&self.session_id)
                ).await?;
                
                // Update token and expiry
                {
                    let mut auth_token = self.auth_token.lock().unwrap();
                    *auth_token = Some(token_result.token);
                    
                    let mut token_expiry = self.token_expiry.lock().unwrap();
                    *token_expiry = Some(Instant::now() + Duration::from_secs(token_result.expires_in));
                }
                
                println!("[refresh_token] Token refreshed successfully");
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Gets the current auth token if available
    pub fn get_token(&self) -> Option<String> {
        self.auth_token.lock().unwrap().clone()
    }

    /// Subscribes the client to a specific topic within its session.
    pub async fn subscribe(&mut self, subscriber_name: &str, topic: &str, payload: &str) {
        println!("[subscribe] subscriber_name={}, topic={}, payload={}, session={}", 
            subscriber_name, topic, payload, self.session_id);
        
        let cmd = format!("subscribe:{}|{}", topic, self.session_id);
        if let Err(e) = self.ws_channel.send(Message::Text(cmd)).await {
            println!("[subscribe] Error: {:?}", e);
        }
    }

    /// Unsubscribes the client from a specific topic within its session.
    pub async fn unsubscribe(&mut self, topic: &str) {
        println!("[unsubscribe] topic={}, session={}", topic, self.session_id);
        let cmd = format!("unsubscribe:{}|{}", topic, self.session_id);
        if let Err(e) = self.ws_channel.send(Message::Text(cmd)).await {
            println!("[unsubscribe] Error: {:?}", e);
        }
    }

    /// Publishes a message to a specific topic within the client's session.
    pub async fn publish(&mut self, publisher_name: &str, topic: &str, payload: &str, timestamp: &str) -> Result<(), String> {
        // Check if token needs refreshing before publishing
        if self.auth_token.lock().unwrap().is_some() {
            if let Err(e) = self.refresh_token_if_needed().await {
                println!("[publish] Error refreshing token: {}", e);
                // Continue anyway with the old token
            }
        }

        // Check connection state first
        if !*self.is_connected.lock().unwrap() {
            return Err("WebSocket is not connected".to_string());
        }

        println!("[publish] publisher_name={}, topic={}, payload={}, timestamp={}, session={}", 
            publisher_name, topic, payload, timestamp, self.session_id);
        
        let msg = json!({
            "publisher_name": publisher_name,
            "topic": topic,
            "payload": payload,
            "timestamp": timestamp,
            "session_id": self.session_id
        });
        let cmd = format!("publish-json:{}", msg.to_string());

        match self.ws_channel.send(Message::Text(cmd)).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Mark as disconnected on error
                *self.is_connected.lock().unwrap() = false;
                Err(format!("Failed to send message: {}", e))
            }
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

    /// Checks if the WebSocket connection is active.
    pub fn is_connected(&self) -> bool {
        *self.is_connected.lock().unwrap()
    }

    /// Checks if the client is authenticated with a JWT token
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.lock().unwrap().is_some()
    }
}
