// web/enc_util.js

/**
 * Generate an ECDH key pair for key exchange
 * @returns {Promise<CryptoKeyPair>} A promise that resolves to a CryptoKeyPair
 */
async function generateKeypair() {
    // Use P-256 curve which is widely supported instead of X25519
    // X25519 is not supported in all browsers yet
    return window.crypto.subtle.generateKey(
        {
            name: "ECDH",
            namedCurve: "P-256"  // Changed from "X25519" to "P-256" for better browser support
        },
        true,
        ["deriveKey", "deriveBits"]
    );
}

/**
 * Export a public key to a base64 string
 * @param {CryptoKey} publicKey - The public key to export
 * @returns {Promise<string>} A promise that resolves to a base64 encoded string
 */
async function exportPublicKey(publicKey) {
    const exported = await window.crypto.subtle.exportKey("raw", publicKey);
    return arrayBufferToBase64(exported);
}

/**
 * Import a base64 encoded public key
 * @param {string} base64PublicKey - Base64 encoded public key
 * @returns {Promise<CryptoKey>} A promise that resolves to a CryptoKey
 */
async function importPublicKey(base64PublicKey) {
    const keyData = base64ToArrayBuffer(base64PublicKey);
    return window.crypto.subtle.importKey(
        "raw",
        keyData,
        {
            name: "ECDH",
            namedCurve: "P-256"
        },
        false,
        []
    );
}

/**
 * Derive a shared secret using our private key and their public key
 * @param {CryptoKey} privateKey - Our private key
 * @param {CryptoKey} publicKey - Their public key
 * @returns {Promise<ArrayBuffer>} A promise that resolves to the shared secret
 */
async function deriveSharedSecret(privateKey, publicKey) {
    return window.crypto.subtle.deriveBits(
        {
            name: "ECDH",
            public: publicKey
        },
        privateKey,
        256  // 256 bits for AES-256
    );
}

/**
 * Encrypt data using AES-GCM with the shared secret
 * @param {ArrayBuffer} data - The data to encrypt
 * @param {ArrayBuffer} sharedSecret - The shared secret
 * @returns {Promise<ArrayBuffer>} A promise that resolves to the encrypted data
 */
async function encrypt(data, sharedSecret) {
    // Generate a random 12-byte nonce/IV
    const nonce = window.crypto.getRandomValues(new Uint8Array(12));
    
    // Import the shared secret as an AES key
    const key = await window.crypto.subtle.importKey(
        "raw",
        sharedSecret,
        { name: "AES-GCM", length: 256 },
        false,
        ["encrypt"]
    );
    
    // Encrypt the data
    const ciphertext = await window.crypto.subtle.encrypt(
        {
            name: "AES-GCM",
            iv: nonce
        },
        key,
        data
    );
    
    // Combine nonce and ciphertext
    const result = new Uint8Array(nonce.length + ciphertext.byteLength);
    result.set(nonce);
    result.set(new Uint8Array(ciphertext), nonce.length);
    
    return result.buffer;
}

/**
 * Decrypt data using AES-GCM with the shared secret
 * @param {ArrayBuffer} encryptedData - The data to decrypt, including the nonce
 * @param {ArrayBuffer} sharedSecret - The shared secret
 * @returns {Promise<ArrayBuffer>} A promise that resolves to the decrypted data
 */
async function decrypt(encryptedData, sharedSecret) {
    // Split nonce and ciphertext
    const nonce = encryptedData.slice(0, 12);
    const ciphertext = encryptedData.slice(12);
    
    // Import the shared secret as an AES key
    const key = await window.crypto.subtle.importKey(
        "raw",
        sharedSecret,
        { name: "AES-GCM", length: 256 },
        false,
        ["decrypt"]
    );
    
    // Decrypt the data
    return window.crypto.subtle.decrypt(
        {
            name: "AES-GCM",
            iv: nonce
        },
        key,
        ciphertext
    );
}

/**
 * Decrypt a payload received from the server
 * @param {string} encryptedBase64 - Base64 encoded encrypted payload
 * @param {ArrayBuffer} sharedSecret - The shared secret used for encryption
 * @returns {Promise<Object>} A promise that resolves to the decrypted JSON object
 */
async function decryptPayload(encryptedBase64, sharedSecret) {
    // Convert the base64 payload to ArrayBuffer
    const encryptedData = base64ToArrayBuffer(encryptedBase64);
    
    // Decrypt the payload
    const decryptedBuffer = await decrypt(encryptedData, sharedSecret);
    
    // Convert the decrypted buffer to a string
    const decryptedText = new TextDecoder().decode(decryptedBuffer);
    
    // Parse the JSON string
    return JSON.parse(decryptedText);
}

/**
 * Convert an ArrayBuffer to a base64 string
 * @param {ArrayBuffer} buffer - The buffer to convert
 * @returns {string} A base64 encoded string
 */
function arrayBufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
}

/**
 * Convert a base64 string to an ArrayBuffer
 * @param {string} base64 - The base64 string to convert
 * @returns {ArrayBuffer} The decoded ArrayBuffer
 */
function base64ToArrayBuffer(base64) {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
}

// Export all functions for use in other modules
export {
    generateKeypair,
    exportPublicKey,
    importPublicKey,
    deriveSharedSecret,
    encrypt,
    decrypt,
    arrayBufferToBase64,
    base64ToArrayBuffer,
    decryptPayload
};
