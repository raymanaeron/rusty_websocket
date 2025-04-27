// src/enc_api_route.rs

use axum::{
    routing::get,
    response::IntoResponse,
    Router,
};
use std::sync::Arc;
use crate::enc_util::{KeyPair, serialize_public_key};

#[derive(Clone)]
pub struct EncApiState {
    pub keypair: Arc<KeyPair>,
}

/// Builds a router exposing encryption-related endpoints
pub fn enc_api_router(state: EncApiState) -> Router {
    Router::new()
        .route("/enc/public-key", get(move || {
            let key = serialize_public_key(&state.keypair.public_key);
            async move { key }
        }))
}
