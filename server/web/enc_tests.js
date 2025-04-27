// web/enc_tests.js
import { generateKeypair, exportPublicKey, importPublicKey, deriveSharedSecret, encrypt, decrypt, decryptPayload } from './enc_utils.js';

// Enhanced log function that writes to both console and HTML
function log(message, type = 'info') {
    console.log(message);
    
    // Try to get the log element after a small delay to ensure DOM is ready
    setTimeout(() => {
        const logElement = document.getElementById('logOutput');
        if (logElement) {
            // Create a new div for the log entry
            const logEntry = document.createElement('div');
            logEntry.className = `log-${type}`;
            logEntry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
            
            // Append to the log element
            logElement.appendChild(logEntry);
            
            // Auto-scroll to the bottom
            logElement.scrollTop = logElement.scrollHeight;
        } else {
            console.warn('Log element not found!');
        }
    }, 0);
}

// Run encryption test
async function runEncTest() {
    log("Starting encryption test...");
    log("Encryption test function called");
    
    try {
        // Generate a key pair for our client
        log("Generating client key pair...");
        const clientKeyPair = await generateKeypair();
        const clientPublicKeyBase64 = await exportPublicKey(clientKeyPair.publicKey);
        log(`Client public key: ${clientPublicKeyBase64.substring(0, 20)}...`);
        
        // Simulate getting the server's public key
        // Update the URL to point to the correct port where the API is hosted
        log("Fetching server public key...");
        const serverPublicKeyResponse = await fetch('http://localhost:8081/enc/public-key');
        const serverPublicKeyBase64 = await serverPublicKeyResponse.text();
        log(`Server public key: ${serverPublicKeyBase64.substring(0, 20)}...`);
        
        // Import the server's public key
        const serverPublicKey = await importPublicKey(serverPublicKeyBase64);
        
        // Derive a shared secret
        log("Deriving shared secret...");
        const sharedSecret = await deriveSharedSecret(clientKeyPair.privateKey, serverPublicKey);
        log("Shared secret derived successfully");
        
        // Test encryption and decryption
        const testMessage = { text: "Hello, secure world!", timestamp: new Date().toISOString() };
        log(`Test message: ${JSON.stringify(testMessage)}`);
        
        // Convert message to ArrayBuffer
        const messageBytes = new TextEncoder().encode(JSON.stringify(testMessage));
        
        // Encrypt the message
        log("Encrypting message...");
        const encryptedData = await encrypt(messageBytes, sharedSecret);
        
        // Decrypt the message
        log("Decrypting message...");
        const decryptedBuffer = await decrypt(encryptedData, sharedSecret);
        const decryptedText = new TextDecoder().decode(decryptedBuffer);
        const decryptedMessage = JSON.parse(decryptedText);
        
        log(`Decrypted message: ${JSON.stringify(decryptedMessage)}`);
        log("Encryption test completed successfully!", "success");
    } catch (error) {
        log(`Error in encryption test: ${error.message}`, "error");
        console.error("Detailed error:", error);
    }
}

// Add script status logs to the HTML
log("enc_tests.js loaded");

// Add event listener for the button
const testButton = document.getElementById('startBtn');
if (testButton) {
    log("Test button found, adding event listener");
    testButton.addEventListener('click', function() {
        log("Button clicked!");
        runEncTest();
    });
} else {
    log("Test button not found. Available buttons:", 
        Array.from(document.querySelectorAll('button'))
            .map(b => `${b.id || 'no-id'}: ${b.textContent}`), 
        "warning");
}

// Also add listener for DOMContentLoaded as a backup
document.addEventListener('DOMContentLoaded', () => {
    log("DOMContentLoaded event fired");
    const testButton = document.getElementById('startBtn');
    if (testButton) {
        log("Test button found in DOMContentLoaded");
        testButton.addEventListener('click', runEncTest);
    } else {
        log("Test button not found in DOMContentLoaded", "warning");
    }
});

// Export the test function for direct use if needed
export { runEncTest };
