# Encryption Compatibility Between Rust and JavaScript

| Item                 | Rust                        | JavaScript                |
|----------------------|----------------------------|----------------------------|
| Key exchange         | p256 crate (P-256 curve)    | WebCrypto (P-256 ECDH)    |
| Symmetric encryption | AES-256-GCM                | AES-256-GCM               |
| Key serialization    | Base64                     | Base64                    |
| Nonce handling       | 12 bytes random            | 12 bytes random           |

The implementations are now directly compatible by using P-256 elliptic curve on both sides. The Rust side can still use x25519-dalek for Rust-to-Rust encryption if desired.

## Key Format Details

### P-256 Keys
- Rust: Uses compressed point format (33 bytes) encoded as Base64
- JavaScript: Uses raw format encoded as Base64
- Both implementations handle the format conversion appropriately
