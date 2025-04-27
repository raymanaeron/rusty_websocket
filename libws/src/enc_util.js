// web/enc_util.js

// Converts ArrayBuffer to Base64 string
function arrayBufferToBase64(buffer) {
    let binary = '';
    const bytes = new Uint8Array(buffer);
    bytes.forEach(b => binary += String.fromCharCode(b));
    return window.btoa(binary);
}

// Converts Base64 string to ArrayBuffer
function base64ToArrayBuffer(base64) {
    const binary = window.atob(base64);
    const buffer = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        buffer[i] = binary.charCodeAt(i);
    }
    return buffer.buffer;
}

// Generates an ephemeral X25519 key pair
export async function generateKeypair() {
    const keys = await window.crypto.subtle.generateKey(
        {
            name: "ECDH",
            namedCurve: "X25519"
        },
        true,
        ["deriveKey", "deriveBits"]
    );
    return keys;
}

// Exports a public key to base64
export async function exportPublicKey(publicKey) {
    const raw = await window.crypto.subtle.exportKey("raw", publicKey);
    return arrayBufferToBase64(raw);
}

// Imports a public key from base64
export async function importPublicKey(base64) {
    const raw = base64ToArrayBuffer(base64);
    return window.crypto.subtle.importKey(
        "raw",
        raw,
        { name: "ECDH", namedCurve: "X25519" },
        true,
        []
    );
}

// Derives a shared symmetric AES-GCM key
export async function deriveSharedKey(privateKey, peerPublicKey) {
    const key = await window.crypto.subtle.deriveKey(
        {
            name: "ECDH",
            public: peerPublicKey
        },
        privateKey,
        {
            name: "AES-GCM",
            length: 256
        },
        false,
        ["encrypt", "decrypt"]
    );
    return key;
}

// Encrypts payload using AES-GCM
export async function encryptPayload(symmetricKey, plaintext) {
    const enc = new TextEncoder();
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const ciphertext = await window.crypto.subtle.encrypt(
        {
            name: "AES-GCM",
            iv: iv
        },
        symmetricKey,
        enc.encode(plaintext)
    );
    const combined = new Uint8Array(iv.length + ciphertext.byteLength);
    combined.set(iv, 0);
    combined.set(new Uint8Array(ciphertext), iv.length);
    return combined;
}

// Decrypts payload using AES-GCM
export async function decryptPayload(symmetricKey, combined) {
    const iv = combined.slice(0, 12);
    const ciphertext = combined.slice(12);
    const decrypted = await window.crypto.subtle.decrypt(
        {
            name: "AES-GCM",
            iv: iv
        },
        symmetricKey,
        ciphertext
    );
    const dec = new TextDecoder();
    return dec.decode(decrypted);
}
