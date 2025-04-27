// src/enc_tests.rs

use p256::{
    ecdh::EphemeralSecret,
    EncodedPoint, PublicKey,
};
use rand::rngs::OsRng;
use reqwest;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use aes_gcm::{
    Aes256Gcm, KeyInit, aead::{Aead, AeadCore},
};
use std::error::Error;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use generic_array::GenericArray;

#[derive(Debug, Serialize, Deserialize)]
struct TestMessage {
    text: String,
    timestamp: String,
}

// Generate a P-256 key pair for client
fn generate_keypair() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random(&mut OsRng);
    let public_key = PublicKey::from(&secret);
    (secret, public_key)
}

// Export public key to base64
fn export_public_key(public_key: &PublicKey) -> String {
    let encoded_point = EncodedPoint::from(*public_key);
    BASE64.encode(encoded_point.compress().as_bytes())
}

// Import base64 public key
fn import_public_key(base64_key: &str) -> Result<PublicKey, Box<dyn Error>> {
    let bytes = BASE64.decode(base64_key)?;
    let point = EncodedPoint::from_bytes(&bytes)?;
    
    PublicKey::from_sec1_bytes(point.as_bytes())
        .map_err(|e| format!("Invalid P-256 public key: {}", e).into())
}

// Derive shared secret
fn derive_shared_secret(private_key: &EphemeralSecret, public_key: &PublicKey) -> Vec<u8> {
    let shared_secret = private_key.diffie_hellman(public_key);
    shared_secret.raw_secret_bytes().to_vec()
}

// Encrypt data using AES-GCM with the shared secret
fn encrypt(data: &[u8], shared_secret: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // Use shared secret as AES key
    let key_bytes = <[u8; 32]>::try_from(shared_secret).map_err(|_| "Invalid key length")?;
    let key = Aes256Gcm::new(GenericArray::from_slice(&key_bytes));
    
    // Generate random nonce - specify the type explicitly
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    // Encrypt the data
    let ciphertext = key.encrypt(&nonce, data)
        .map_err(|e| format!("Encryption error: {:?}", e))?;
    
    // Combine nonce and ciphertext
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

// Decrypt data using AES-GCM with the shared secret
fn decrypt(encrypted_data: &[u8], shared_secret: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() <= 12 {
        return Err("Encrypted data too short".into());
    }
    
    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    
    // Create a typed nonce for Aes256Gcm
    let nonce = GenericArray::from_slice(nonce_bytes);
    
    // Use shared secret as AES key
    let key_bytes = <[u8; 32]>::try_from(shared_secret).map_err(|_| "Invalid key length")?;
    let key = Aes256Gcm::new(GenericArray::from_slice(&key_bytes));
    
    // Decrypt the data
    let plaintext = key.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption error: {:?}", e))?;
    
    Ok(plaintext)
}

// Get current timestamp in ISO format
fn get_timestamp() -> String {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = since_epoch.as_secs();
    let millis = since_epoch.subsec_millis();
    
    // Very simple ISO timestamp format
    let dt = time::OffsetDateTime::from_unix_timestamp(secs as i64).unwrap();
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", 
        dt.year(), dt.month() as u8, dt.day(), 
        dt.hour(), dt.minute(), dt.second(), millis)
}

// This function will run the encryption tests that match the JavaScript tests
pub async fn run_encryption_tests() -> Result<(), Box<dyn Error>> {
    println!("Running encryption tests from Rust client...");
    
    // Generate client key pair
    println!("Generating client key pair...");
    let (client_private_key, client_public_key) = generate_keypair();
    let client_public_key_base64 = export_public_key(&client_public_key);
    println!("Client public key: {}...", &client_public_key_base64[..20]);
    
    // Fetch server's public key
    println!("Fetching server public key...");
    let server_public_key_response = reqwest::get("http://127.0.0.1:8082/enc/public-key").await?;
    let server_public_key_base64 = server_public_key_response.text().await?;
    println!("Server public key: {}...", &server_public_key_base64[..20]);
    
    // Import server's public key
    let server_public_key = import_public_key(&server_public_key_base64)?;
    
    // Derive shared secret
    println!("Deriving shared secret...");
    let shared_secret = derive_shared_secret(&client_private_key, &server_public_key);
    println!("Shared secret derived successfully");
    
    // Create test message (matching JavaScript test)
    let test_message = TestMessage {
        text: "Hello, secure world!".to_string(),
        timestamp: get_timestamp(),
    };
    println!("Test message: {:?}", test_message);
    
    // Serialize the message to JSON
    let message_json = serde_json::to_string(&test_message)?;
    let message_bytes = message_json.as_bytes();
    
    // Encrypt the message
    println!("Encrypting message...");
    let encrypted_data = encrypt(message_bytes, &shared_secret)?;
    
    // Decrypt the message
    println!("Decrypting message...");
    let decrypted_bytes = decrypt(&encrypted_data, &shared_secret)?;
    let decrypted_json = String::from_utf8(decrypted_bytes)?;
    let decrypted_message: TestMessage = serde_json::from_str(&decrypted_json)?;
    
    println!("Decrypted message: {:?}", decrypted_message);
    println!("Encryption test completed successfully!");
    
    Ok(())
}
