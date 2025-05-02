use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Claims structure for JWT tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user identifier)
    pub sub: String,
    /// Session ID to link with existing session mechanics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    /// Issued at time
    pub iat: u64,
    /// Expiration time
    pub exp: u64,
}

/// Creates a new JWT token
pub fn create_token(
    user_id: &str,
    session_id: Option<&str>,
    secret: &[u8],
    expiration: Duration,
) -> Result<String, Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    let claims = Claims {
        sub: user_id.to_string(),
        sid: session_id.map(|s| s.to_string()),
        iat: now,
        exp: now + expiration.as_secs(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;

    Ok(token)
}

/// Validates and decodes a JWT token
pub fn validate_token(token: &str, secret: &[u8]) -> Result<Claims, Box<dyn Error>> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}

/// Extracts token from various formats
pub fn extract_token(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}
