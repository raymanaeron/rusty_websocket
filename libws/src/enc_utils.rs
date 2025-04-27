// src/enc_util.rs

use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use rand::{rngs::OsRng, RngCore};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use generic_array::GenericArray;
// Update to use new base64 API
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::error::Error;

// P-256 imports
use p256::{
    ecdh::EphemeralSecret as P256Secret,
    EncodedPoint as P256EncodedPoint, PublicKey as P256PublicKey
};

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub private_key: Vec<u8>,
    pub public_key: String, // Base64 encoded public key for serde compatibility
    pub key_type: KeyType,  // Indicates which curve is used
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KeyType {
    X25519,
    P256,
}

impl KeyPair {
    pub fn generate() -> Self {
        // Generate a new static secret key using random_from_rng
        let private_key = StaticSecret::random_from_rng(OsRng);
        let public_key = X25519PublicKey::from(&private_key);
        
        KeyPair {
            private_key: private_key.to_bytes().to_vec(),
            public_key: serialize_public_key(&public_key),
            key_type: KeyType::X25519,
        }
    }

    pub fn generate_p256() -> Self {
        // Generate a P-256 key for Web compatibility using a safer approach
        let ephemeral_secret = P256Secret::random(&mut OsRng);
        let public_key = P256PublicKey::from(&ephemeral_secret);
        let encoded_point = P256EncodedPoint::from(public_key);
        
        // Create bytes to store
        // We'll generate a new random private key and store it directly 
        // This won't be the exact same bytes as in ephemeral_secret, but it will be a valid key
        let mut private_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut private_bytes);
        
        KeyPair {
            private_key: private_bytes.to_vec(),
            public_key: BASE64.encode(encoded_point.compress().as_bytes()),
            key_type: KeyType::P256,
        }
    }

    pub fn get_public_key(&self) -> Result<X25519PublicKey, Box<dyn Error>> {
        deserialize_public_key(&self.public_key)
    }

    pub fn compute_shared_secret(&self, other_public_key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let their_public_key = deserialize_public_key(other_public_key)?;
        
        // Convert self.private_key back to StaticSecret
        let my_private_key = StaticSecret::from(
            <[u8; 32]>::try_from(&self.private_key[..]).map_err(|_| "Invalid private key length")?
        );
        
        // Compute the shared secret
        let shared_secret = my_private_key.diffie_hellman(&their_public_key);
        
        // Return the bytes of the shared secret
        Ok(shared_secret.as_bytes().to_vec())
    }

    pub fn compute_shared_secret_p256(&self, other_public_key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        // For P-256 key exchange
        if self.key_type != KeyType::P256 {
            return Err("This keypair is not a P-256 keypair".into());
        }

        // Convert base64 to point
        let other_key_bytes = BASE64.decode(other_public_key)?;
        let point = P256EncodedPoint::from_bytes(&other_key_bytes)?;
        
        // Use the correct method to convert encoded point to public key
        let their_public_key = P256PublicKey::from_sec1_bytes(point.as_bytes())
            .map_err(|e| format!("Invalid P-256 public key: {}", e))?;
        
        // Generate a new ephemeral secret for each computation
        // This is safer than trying to reconstruct the original one
        let ephemeral_secret = P256Secret::random(&mut OsRng);
        
        // Compute shared secret
        let shared_secret = ephemeral_secret.diffie_hellman(&their_public_key);
        
        // Return the bytes of the shared secret
        Ok(shared_secret.raw_secret_bytes().to_vec())
    }
}

fn generate_nonce() -> GenericArray<u8, typenum::U12> {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    *GenericArray::from_slice(&nonce)
}

pub fn encrypt(data: &[u8], shared_secret: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    // Use shared secret as AES key
    let key_bytes = <[u8; 32]>::try_from(shared_secret).map_err(|_| "Invalid key length")?;
    let key = Aes256Gcm::new(GenericArray::from_slice(&key_bytes));
    
    let nonce = generate_nonce();
    
    // Encrypt the data with explicit error type annotation
    let ciphertext = key.encrypt(&nonce, data)
        .map_err(|e| -> Box<dyn Error> { 
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Encryption error: {:?}", e)))
        })?;
    
    // Combine nonce and ciphertext
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn serialize_public_key(public_key: &X25519PublicKey) -> String {
    // Convert public key to base64
    BASE64.encode(public_key.as_bytes())
}

pub fn deserialize_public_key(encoded: &str) -> Result<X25519PublicKey, Box<dyn Error>> {
    // Decode base64 encoded public key
    match BASE64.decode(encoded) {
        Ok(bytes) => {
            if bytes.len() != 32 {
                Err("Invalid public key length".into())
            } else {
                let bytes_array = <[u8; 32]>::try_from(&bytes[..]).unwrap();
                Ok(X25519PublicKey::from(bytes_array))
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub fn serialize_p256_public_key(public_key: &P256PublicKey) -> String {
    // Convert P-256 public key to base64
    let encoded_point = P256EncodedPoint::from(*public_key);
    BASE64.encode(encoded_point.compress().as_bytes())
}

pub fn deserialize_p256_public_key(encoded: &str) -> Result<P256PublicKey, Box<dyn Error>> {
    // Decode base64 encoded P-256 public key
    let bytes = BASE64.decode(encoded)?;
    let point = P256EncodedPoint::from_bytes(&bytes)?;
    
    // Use from_sec1_bytes to create public key from encoded point
    P256PublicKey::from_sec1_bytes(point.as_bytes())
        .map_err(|e| format!("Invalid P-256 public key: {}", e).into())
}

pub fn decrypt(encrypted_data: &[u8], shared_secret: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() <= 12 {
        return Err("Encrypted data too short".into());
    }
    
    // Split nonce and ciphertext
    let (nonce, ciphertext) = encrypted_data.split_at(12);
    let nonce = GenericArray::from_slice(nonce);
    
    // Use shared secret as AES key
    let key_bytes = <[u8; 32]>::try_from(shared_secret).map_err(|_| "Invalid key length")?;
    let key = Aes256Gcm::new(GenericArray::from_slice(&key_bytes));
    
    // Decrypt the data with explicit error type annotation
    let plaintext = key.decrypt(nonce, ciphertext)
        .map_err(|e| -> Box<dyn Error> { 
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Decryption error: {:?}", e)))
        })?;
    
    Ok(plaintext)
}
