// src/main.rs
use axum::{
    Router,
    routing::get,
    extract::{
        connect_info::ConnectInfo, 
        ws::WebSocketUpgrade,
        State,
        Query,
    },
    response::IntoResponse,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use libws::{Subscribers, WebSocketParams};
mod ws_tests; // Updated from client_tests
mod enc_tests;

use std::{
    collections::HashMap,
    env,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::cors::{Any, CorsLayer};
use libws::enc_api_route::{enc_api_router, create_web_compatible_state};
use libws::jwt_api_route::{jwt_api_router, create_default_jwt_state}; // Add the JWT API module

/// Adapter function to bridge between server and library
async fn handle_socket_adapter(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(subscribers): State<Subscribers>,
    query_params: Option<Query<WebSocketParams>>,  // Add query parameters
) -> impl IntoResponse {
    // Call the libws handler with query parameters
    libws::handle_socket(ws, ConnectInfo(addr), query_params, subscribers).await
}

#[tokio::main]
async fn main() {
    // Set a custom panic hook to log panic information
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("[server] PANIC: {:?}", panic_info);
    }));

    // Log environment variable configuration for JWT
    if env::var("JWT_SECRET_KEY").is_ok() {
        println!("Using JWT_SECRET_KEY from environment");
    } else {
        println!("JWT_SECRET_KEY not set - using default (insecure for production)");
    }

    if let Ok(expiration) = env::var("JWT_EXPIRATION_SECONDS") {
        println!("Using JWT_EXPIRATION_SECONDS: {} seconds", expiration);
    } else {
        println!("JWT_EXPIRATION_SECONDS not set - using default (3600 seconds)");
    }

    // Parse command-line arguments to determine the mode of operation
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--web" {
        run_web_test().await; // Run the web test mode
    } else {
        run_local_test().await; // Run the local test mode
    }
}

/// Runs the server in web test mode, serving both WebSocket and static web content.
async fn run_web_test() {
    // Initialize the subscribers map with session support
    let subscribers: Subscribers = Arc::new(Mutex::new(HashMap::new()));

    // Generate a web-compatible keypair for encryption tests
    let enc_state = create_web_compatible_state();
    
    // Create JWT state for authentication
    let jwt_state = create_default_jwt_state();

    // Setup CORS for the API
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create encryption router with the same state type as the main router
    let encryption_router = enc_api_router::<Subscribers>(enc_state);
    
    // Create JWT authentication router
    let jwt_router = jwt_api_router::<Subscribers>(jwt_state);

    // Configure the WebSocket app on port 8081
    let ws_app = Router::new()
        .route(
            "/ws",
            get(handle_socket_adapter),
        )
        // Now merge both routers
        .merge(encryption_router)
        .merge(jwt_router) // Add the JWT router
        .layer(cors)
        .with_state(subscribers.clone());

    // Spawn a task to handle WebSocket connections
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
        println!("Listening at ws://127.0.0.1:8081/ws");
        println!("Encryption API available at http://127.0.0.1:8081/enc/public-key");
        println!("JWT API available at http://127.0.0.1:8081/jwt"); // Add JWT API info
        axum::serve(listener, ws_app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    // Configure the static web app on port 8080
    let web_app = Router::new()
        .nest_service("/", ServeDir::new("web"));

    // Serve the static web content
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Serving web UI at http://127.0.0.1:8080");

    axum::serve(listener, web_app.into_make_service())
        .await
        .unwrap();
}

/// Runs the server in local test mode, first running encryption tests followed by WebSocket tests.
async fn run_local_test() {
    println!("Starting local test sequence...");
    
    // First run the encryption tests
    run_local_enc_tests().await;
    
    // Then run the WebSocket tests
    run_local_ws_tests().await;
    
    println!("All local tests completed.");
}

/// Runs local encryption tests
async fn run_local_enc_tests() {
    println!("\n=== Starting Encryption Tests ===");
    
    // Generate a web-compatible keypair for encryption tests
    let enc_state = create_web_compatible_state();
    
    // Create JWT state for authentication
    let jwt_state = create_default_jwt_state();
    
    // Setup CORS for the API
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
        
    // Create encryption router with dummy state since it's not needed for tests
    let encryption_router = enc_api_router::<()>(enc_state);
    
    // Create JWT authentication router
    let jwt_router = jwt_api_router::<()>(jwt_state);
    
    // Configure the encryption API server on port 8082 (different port to avoid conflicts)
    let enc_app = Router::new()
        .merge(encryption_router)
        .merge(jwt_router) // Add the JWT router
        .layer(cors);
    
    // Start the encryption API server
    let listener = TcpListener::bind("127.0.0.1:8082").await.unwrap();
    println!("Encryption API available at http://127.0.0.1:8082/enc/public-key");
    println!("JWT API available at http://127.0.0.1:8082/jwt"); // Add JWT API info
    
    // Start the server in a background task
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, enc_app.into_make_service())
            .await
            .unwrap();
    });
    
    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // Run the encryption tests that match the JavaScript tests
    match enc_tests::run_encryption_tests().await {
        Ok(_) => println!("✓ Encryption tests passed successfully"),
        Err(e) => println!("✗ Encryption tests failed: {}", e),
    };
    
    // Terminate the server after tests
    server_handle.abort();
    println!("=== Encryption Tests Completed ===\n");
}

/// Runs local WebSocket tests (previously the content of run_local_test)
async fn run_local_ws_tests() {
    println!("=== Starting WebSocket Tests ===");
    
    // Initialize the subscribers map with session support
    let subscribers: Subscribers = Arc::new(Mutex::new(HashMap::new()));

    // Configure the WebSocket app on port 8081
    let app = Router::new().route(
        "/ws",
        get(handle_socket_adapter),
    ).with_state(subscribers.clone());

    // Start the WebSocket server
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("Listening at ws://127.0.0.1:8081/ws");

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    // Run client tests after a slight delay to let the server start
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    ws_tests::run_client_tests().await; // Updated from client_tests to ws_tests
    
    // Terminate the server after tests
    server_handle.abort();
    println!("=== WebSocket Tests Completed ===");
}
