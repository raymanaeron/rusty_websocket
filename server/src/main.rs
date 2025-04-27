// src/main.rs
use axum::{routing::get, Router};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use libws::{handle_socket, Subscribers};
mod client_tests;

use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Set a custom panic hook to log panic information
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("[server] PANIC: {:?}", panic_info);
    }));

    // Parse command-line arguments to determine the mode of operation
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--web" {
        run_web_test().await; // Run the web test mode
    } else {
        run_local_test().await; // Run the local test mode
    }
}

/// Runs the server in web test mode, serving both WebSocket and static web content.
/// The WebSocket server now supports session-based topic subscriptions, which ensures
/// that messages are only delivered to subscribers within the same session.
async fn run_web_test() {
    // Initialize the subscribers map with session support
    let subscribers: Subscribers = Arc::new(Mutex::new(HashMap::new()));

    // Configure the WebSocket app on port 8081
    let ws_app = Router::new().route(
        "/ws",
        get({
            let subs = subscribers.clone();
            move |ws, info| handle_socket(ws, info, subs.clone())
        }),
    );

    // Spawn a task to handle WebSocket connections
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
        println!("Listening at ws://127.0.0.1:8081/ws");
        axum::serve(listener, ws_app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    // Configure the static web app on port 8080
    let web_app = Router::new().nest_service("/", ServeDir::new("web"));

    // Serve the static web content
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Serving web UI at http://127.0.0.1:8080");

    axum::serve(listener, web_app.into_make_service())
        .await
        .unwrap();
}

/// Runs the server in local test mode, including WebSocket handling and client tests.
async fn run_local_test() {
    // Initialize the subscribers map with session support
    let subscribers: Subscribers = Arc::new(Mutex::new(HashMap::new()));

    // Configure the WebSocket app on port 8081
    let app = Router::new().route(
        "/ws",
        get({
            let subs = subscribers.clone();
            move |ws, info| handle_socket(ws, info, subs.clone())
        }),
    );

    // Start the WebSocket server
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("Listening at ws://127.0.0.1:8081/ws");

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });

    // Run client tests after a slight delay to let the server start
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    client_tests::run_client_tests().await;
}
