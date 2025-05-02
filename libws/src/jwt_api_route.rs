use axum::{
    Router,
    routing::post,
    extract::State,
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use std::env;
use crate::jwt_utils::create_token;

/// JWT configuration state
#[derive(Clone)]
pub struct JwtState {
    pub secret_key: Arc<[u8; 32]>,
    pub token_expiration: Duration,
}

/// Request payload for authentication
#[derive(Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub session_id: Option<String>,
}

/// Response payload for successful authentication
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
}

/// Error response for failed authentication
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// Define a unified API response to handle both success and error cases
enum ApiResponse {
    Success(AuthResponse),
    Error(StatusCode, ErrorResponse),
}

// Implement IntoResponse for our custom API response
impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Success(response) => {
                (StatusCode::OK, Json(response)).into_response()
            }
            ApiResponse::Error(status, response) => {
                (status, Json(response)).into_response()
            }
        }
    }
}

/// Creates a router with JWT authentication endpoints
pub fn jwt_api_router<S>(state: JwtState) -> Router<S> 
where 
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/auth/token", post(
            move |State(_): State<S>, Json(auth_request): Json<AuthRequest>| async move {
                // This is a simple authentication mechanism for demo purposes
                // In a real application, you would validate credentials against a database
                if auth_request.username.is_empty() || auth_request.password.is_empty() {
                    return ApiResponse::Error(
                        StatusCode::UNAUTHORIZED, 
                        ErrorResponse {
                            error: "Invalid credentials".to_string(),
                        }
                    );
                }

                // Create JWT token
                match create_token(
                    &auth_request.username, 
                    auth_request.session_id.as_deref(), 
                    &state.secret_key[..],
                    state.token_expiration
                ) {
                    Ok(token) => {
                        ApiResponse::Success(AuthResponse {
                            token,
                            expires_in: state.token_expiration.as_secs(),
                        })
                    },
                    Err(_) => {
                        ApiResponse::Error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ErrorResponse {
                                error: "Failed to generate token".to_string(),
                            }
                        )
                    }
                }
            }
        ))
}

/// Creates a JWT state with reasonable defaults
pub fn create_default_jwt_state() -> JwtState {
    // Create a default secret key
    let mut secret_key = [0u8; 32];
    
    // Try to get JWT secret from environment variable
    match env::var("JWT_SECRET_KEY") {
        Ok(env_key) => {
            // Copy bytes from environment variable, up to 32 bytes
            let bytes = env_key.as_bytes();
            for i in 0..std::cmp::min(bytes.len(), 32) {
                secret_key[i] = bytes[i];
            }
        },
        Err(_) => {
            // Use default key
            eprintln!("WARNING: Using default JWT secret key. This is insecure for production!");
            eprintln!("Set the JWT_SECRET_KEY environment variable for better security.");
            
            let default_bytes = b"rusty_websocket_jwt_secret_key_32b";
            for i in 0..32 {
                secret_key[i] = default_bytes[i];
            }
        }
    }
    
    // Use default expiration of 1 hour (3600 seconds)
    let default_expiration = 3600;
    let mut expiration_seconds = default_expiration;
    
    // Try to get expiration from environment variable
    if let Ok(val) = env::var("JWT_EXPIRATION_SECONDS") {
        if let Ok(seconds) = val.parse::<u64>() {
            expiration_seconds = seconds;
        } else {
            eprintln!("WARNING: Invalid JWT_EXPIRATION_SECONDS value, using default (3600)");
        }
    }
    
    JwtState {
        secret_key: Arc::new(secret_key),
        token_expiration: Duration::from_secs(expiration_seconds),
    }
}
