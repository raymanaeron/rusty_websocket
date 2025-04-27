# Encryption Compatibility Between Rust and JavaScript

| Item                 | Rust                  | JavaScript                   |
|----------------------|------------------------|-------------------------------|
| Key exchange         | x25519-dalek           | WebCrypto (X25519 ECDH)       |
| Symmetric encryption | AES-256-GCM            | AES-256-GCM                   |
| Key serialization    | Base64                 | Base64                        |
| Nonce handling       | 12 bytes random        | 12 bytes random               |
