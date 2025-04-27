// src/enc_api_route.rs

use axum::{
    Router,
    routing::get,
    extract::State,
};
use crate::enc_utils::KeyPair;
use std::sync::Arc;

#[derive(Clone)]
pub struct EncApiState {
    pub keypair: Arc<KeyPair>,
}

/// Builds a router exposing encryption-related endpoints
/// The generic parameter allows the router to be compatible with different state types
pub fn enc_api_router<S>(state: EncApiState) -> Router<S> 
where 
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/enc/public-key", get(
            move |_: State<S>| async move {
                // Just return the stored base64 public key directly
                state.keypair.public_key.clone()
            }
        ))
}

/// Create a new EncApiState with a P-256 keypair for web compatibility
pub fn create_web_compatible_state() -> EncApiState {
    let keypair = Arc::new(KeyPair::generate_p256());
    println!("Generated web-compatible P-256 encryption key");
    EncApiState { keypair }
}
