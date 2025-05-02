// JWT WebSocket Authentication Test

import { jwtManager, createAuthenticatedWebSocket } from './jwt_utils.js';

// Enhanced log function that writes to both console and HTML
function log(message, type = 'info') {
    console.log(message);
    
    // Try to get the log element
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
}

// Run the JWT WebSocket test
async function runJwtWebSocketTest() {
    log("Starting JWT WebSocket authentication test...");

    try {
        const sessionId = `jwt-session-${Date.now()}`;
        const authUrl = 'http://localhost:8081/auth/token';
        const wsUrl = 'ws://127.0.0.1:8081/ws';
        const username = 'testuser';
        const password = 'password';
        
        log(`Authenticating user: ${username} with session: ${sessionId}`, 'info');
        
        // Create an authenticated WebSocket connection
        const ws = await createAuthenticatedWebSocket(
            wsUrl, 
            authUrl, 
            username, 
            password, 
            sessionId
        );
        
        log("Authentication successful", 'success');
        
        // Add error handler for better debugging
        ws.onerror = (error) => {
            log(`WebSocket error: ${error}`, 'error');
            console.error('WebSocket error details:', error);
        };
        
        // Enhanced message handler with more detailed logging
        ws.onmessage = (event) => {
            try {
                log(`Raw message received: ${event.data}`, 'info');
                
                if (event.data === "pong") {
                    log("Received pong response from server", 'success');
                    return;
                }
                
                try {
                    const data = JSON.parse(event.data);
                    log(`Received message: Topic=${data.topic}, Payload=${data.payload}`, 'success');
                    log(`Message details: Publisher=${data.publisher_name}, Session=${data.session_id}, Timestamp=${data.timestamp}`, 'info');
                } catch (parseError) {
                    log(`Received non-JSON message: ${event.data}`, 'info');
                }
            } catch (error) {
                log(`Error handling message: ${error.message}`, 'error');
            }
        };

        // Subscribe to a topic for testing - always include explicit session ID
        const testTopic = "AuthenticatedTestTopic";
        log(`Subscribing to topic: ${testTopic} with explicit session ID: ${sessionId}`);
        ws.send(`subscribe:${testTopic}|${sessionId}`);
        log(`Subscribe request sent for topic: ${testTopic}`, 'info');

        // Wait a moment, then publish a message to the topic
        setTimeout(() => {
            const message = {
                publisher_name: username,
                topic: testTopic,
                payload: "Hello from authenticated client!",
                timestamp: new Date().toISOString(),
                session_id: sessionId
            };
            
            log(`Publishing message to ${testTopic}: "${message.payload}"`, 'info');
            ws.send(`publish-json:${JSON.stringify(message)}`);
            log(`Full JSON payload: ${JSON.stringify(message)}`, 'info');
        }, 1000);
        
        // Add additional test with ping/pong after 2 seconds
        setTimeout(() => {
            log("Checking if message was received back from server...", 'info');
            if (!document.querySelector('.log-success')) {
                log("No message received back from server after publishing. This might indicate a problem.", 'warning');
                log("Sending ping message to test basic messaging...", 'info');
                ws.send("ping");
            }
        }, 2000);
        
        // Try alternative subscription with explicit session ID after 3 seconds
        setTimeout(() => {
            log("Trying subscription with explicit session ID...", 'info');
            ws.send(`subscribe:${testTopic}|${sessionId}`);
            
            // Wait a moment then publish again
            setTimeout(() => {
                log("Trying alternative message with explicit session ID...", 'info');
                const altMessage = {
                    publisher_name: username,
                    topic: testTopic,
                    payload: "Follow-up message with explicit session",
                    timestamp: new Date().toISOString(),
                    session_id: sessionId
                };
                ws.send(`publish-json:${JSON.stringify(altMessage)}`);
            }, 500);
        }, 3000);
        
        // Verify token is valid and will be refreshed
        const timeUntilRefresh = jwtManager.expiresAt - Date.now();
        log(`Token will be refreshed in ${Math.round(timeUntilRefresh/1000)} seconds`, 'info');
        
        // Add close handler
        ws.onclose = (event) => {
            log(`WebSocket connection closed: Code=${event.code}, Reason=${event.reason || 'No reason provided'}`, 'warning');
        };
        
        // Add a button to close the connection
        const closeButton = document.createElement('button');
        closeButton.textContent = 'Close Connection';
        closeButton.onclick = () => {
            ws.close();
            jwtManager.clearToken();
            log("Connection manually closed", 'info');
        };
        
        // Add a button to test the connection
        const testConnBtn = document.createElement('button');
        testConnBtn.textContent = 'Test Connection';
        testConnBtn.onclick = () => {
            log("Testing connection with echo message...", 'info');
            ws.send(`publish-json:${JSON.stringify({
                publisher_name: username,
                topic: testTopic,
                payload: "Echo test " + new Date().toISOString(),
                timestamp: new Date().toISOString(),
                session_id: sessionId
            })}`);
        };
        
        const actionsDiv = document.querySelector('.actions');
        if (actionsDiv) {
            actionsDiv.appendChild(closeButton);
            actionsDiv.appendChild(testConnBtn);
        }
        
    } catch (error) {
        log(`Error in JWT WebSocket test: ${error.message}`, 'error');
        console.error("Full error:", error);
    }
}

// Add script status logs
log("jwt_websocket_test.js loaded");

// Add event listener for the button
const testButton = document.getElementById('startBtn');
if (testButton) {
    log("Test button found, adding event listener");
    testButton.addEventListener('click', runJwtWebSocketTest);
} else {
    log("Test button not found", "warning");
}

// Also add listener for DOMContentLoaded as a backup
document.addEventListener('DOMContentLoaded', () => {
    log("DOMContentLoaded event fired");
    const testButton = document.getElementById('startBtn');
    if (testButton) {
        log("Test button found in DOMContentLoaded");
        testButton.addEventListener('click', runJwtWebSocketTest);
    }
});

// Export the test function for direct use if needed
export { runJwtWebSocketTest };
