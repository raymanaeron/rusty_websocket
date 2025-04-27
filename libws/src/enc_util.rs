// src/enc_util.rs

use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit, OsRng, generic_array::GenericArray, Payload}};
use rand_core::RngCore;
use base64::{encode as b64_encode, decode as b64_decode};

/// Represents a generated key pair
pub struct KeyPair {
    pub private_key: StaticSecret,
    pub public_key: X25519PublicKey,
}

/// Represents a derived symmetric key
pub struct SymmetricKey {
    key: Aes256Gcm,
}

impl SymmetricKey {
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let nonce = Self::generate_nonce();
        let payload = Payload { msg: plaintext, aad: &[] };
        self.key.encrypt(&nonce, payload)
            .map(|mut ciphertext| {
                // Prepend nonce to ciphertext for later decryption
                let mut combined = nonce.to_vec();
                combined.append(&mut ciphertext);
                combined
            })
            .map_err(|e| format!("Encryption failed: {:?}", e))
    }

    pub fn decrypt(&self, combined: &[u8]) -> Result<Vec<u8>, String> {
        if combined.len() < 12 {
            return Err("Invalid ciphertext".to_string());
        }
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = GenericArray::from_slice(nonce_bytes);
        self.key.decrypt(nonce, Payload { msg: ciphertext, aad: &[] })
            .map_err(|e| format!("Decryption failed: {:?}", e))
    }

    fn generate_nonce() -> GenericArray<u8, typenum::U12> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        GenericArray::clone_from_slice(&nonce_bytes)
    }
}

/// Generates a new ephemeral key pair
pub fn generate_keypair() -> KeyPair {
    let private_key = StaticSecret::new(OsRng);
    let public_key = X25519PublicKey::from(&private_key);
    KeyPair { private_key, public_key }
}

/// Serializes a public key into a base64 string
pub fn serialize_public_key(public_key: &X25519PublicKey) -> String {
    b64_encode(public_key.as_bytes())
}

/// Deserializes a base64 string into a public key
pub fn deserialize_public_key(encoded: &str) -> Result<X25519PublicKey, String> {
    b64_decode(encoded)
        .map_err(|e| format!("Base64 decode failed: {:?}", e))
        .and_then(|bytes| {
            if bytes.len() != 32 {
                Err("Invalid public key length".to_string())
            } else {
                Ok(X25519PublicKey::from(bytes.as_slice()))
            }
        })
}

/// Derives a symmetric session key from our private key and peer's public key
pub fn derive_shared_key(my_private_key: &StaticSecret, peer_public_key: &X25519PublicKey) -> SymmetricKey {
    let shared_secret = my_private_key.diffie_hellman(peer_public_key);
    let shared_bytes = shared_secret.as_bytes();
    let key = Aes256Gcm::new(GenericArray::clone_from_slice(shared_bytes));
    SymmetricKey { key }
}
