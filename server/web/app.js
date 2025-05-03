import { jwtManager, createAuthenticatedWebSocket } from './jwt_utils.js';

// Enhanced log function that writes to both console and HTML
function log(message, type = 'info') {
    console.log(message);
    
    const logElement = document.getElementById('logOutput');
    if (logElement) {
        const logEntry = document.createElement('div');
        logEntry.className = `log-${type}`;
        logEntry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
        
        logElement.appendChild(logEntry);
        logElement.scrollTop = logElement.scrollHeight;
    }
}

// Check if WebSocket is supported
function checkWebSocketSupport() {
    if (window.WebSocket) {
        log("WebSocket is supported by your browser", "success");
        return true;
    } else {
        log("WebSocket is NOT supported by your browser", "error");
        return false;
    }
}

// Basic connectivity check
async function checkConnectivity() {
    log("Checking WebSocket connectivity...");
    
    if (!checkWebSocketSupport()) {
        return;
    }
    
    try {
        // Try to establish a simple WebSocket connection
        const ws = new WebSocket("ws://127.0.0.1:8081/ws");
        
        ws.onopen = function() {
            log("WebSocket connection established successfully", "success");
            log("Sending a message: ping", "success");
            ws.send("ping");
        };
        
        ws.onerror = function(error) {
            log(`WebSocket connection error: ${error}`, "error");
            console.error("WebSocket error details:", error);
        };
        
        ws.onmessage = function(event) {
            log(`Received message: ${event.data}`, "info");
            // Close the connection after we get the response
            ws.close();
        };
        
        ws.onclose = function() {
            log("WebSocket connection closed", "info");
        };
    } catch (error) {
        log(`Error checking connectivity: ${error.message}`, "error");
        console.error("Full error:", error);
    }
}

// Run the full test based on the specified use case
async function runTest() {
    log("Starting WebSocket pub/sub test...");
    
    if (!checkWebSocketSupport()) {
        return;
    }
    
    try {
        // Define event topics
        const detectEvent = "DetectCustomerEvent";
        const connectEvent = "NetworkConnectedEvent";
        const registrationEvent = "RegistrationCompleteEvent";
        
        // Define two distinct sessions
        const sessionA = "session-A";
        const sessionB = "session-B";
        
        // Define authentication details
        const authUrl = 'http://localhost:8081/auth/token';
        const wsUrl = 'ws://127.0.0.1:8081/ws';
        const username = 'testuser';
        const password = 'password';
        
        // Show the close connection button
        const closeConnectionBtn = document.getElementById('closeConnectionBtn');
        if (closeConnectionBtn) {
            closeConnectionBtn.style.display = 'inline-block';
        }
        
        log("Setting up Client1 in session-A...");
        const client1 = await setupClient("Client1", sessionA, wsUrl, authUrl, username, password);
        
        log("Setting up Client2 in session-A...");
        const client2 = await setupClient("Client2", sessionA, wsUrl, authUrl, username, password);
        
        log("Setting up Client3 in session-B...");
        const client3 = await setupClient("Client3", sessionB, wsUrl, authUrl, username, password);
        
        log("Setting up Client4 in session-B...");
        const client4 = await setupClient("Client4", sessionB, wsUrl, authUrl, username, password);
        
        // Set up subscriptions for each client
        log("Setting up subscriptions...");
        
        // Client1 subscriptions
        client1.ws.send(`subscribe:${detectEvent}|${sessionA}`);
        client1.ws.send(`subscribe:${connectEvent}|${sessionA}`);
        log("Client1 subscribed to DetectCustomerEvent and NetworkConnectedEvent in session-A");
        
        // Client2 subscriptions
        client2.ws.send(`subscribe:${detectEvent}|${sessionA}`);
        client2.ws.send(`subscribe:${registrationEvent}|${sessionA}`);
        log("Client2 subscribed to DetectCustomerEvent and RegistrationCompleteEvent in session-A");
        
        // Client3 subscriptions
        client3.ws.send(`subscribe:${detectEvent}|${sessionB}`);
        client3.ws.send(`subscribe:${connectEvent}|${sessionB}`);
        log("Client3 subscribed to DetectCustomerEvent and NetworkConnectedEvent in session-B");
        
        // Client4 subscriptions
        client4.ws.send(`subscribe:${registrationEvent}|${sessionB}`);
        client4.ws.send(`subscribe:${connectEvent}|${sessionB}`);
        log("Client4 subscribed to RegistrationCompleteEvent and NetworkConnectedEvent in session-B");
        
        // Allow some time for subscriptions to propagate
        log("Waiting for subscriptions to propagate...", "info");
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Start publishing messages
        log("Publishing messages...", "success");
        
        // Generate timestamp for messages
        const timestamp = new Date().toISOString();
        
        // Client1 publishes RegistrationCompleteEvent in session-A
        const client1Message = {
            publisher_name: "Client1",
            topic: registrationEvent,
            payload: "Registration complete from session A",
            timestamp: timestamp,
            session_id: sessionA
        };
        client1.ws.send(`publish-json:${JSON.stringify(client1Message)}`);
        log("Client1 published RegistrationCompleteEvent in session-A");
        
        // Client2 publishes NetworkConnectedEvent in session-A
        const client2Message = {
            publisher_name: "Client2",
            topic: connectEvent,
            payload: "Network connected in session A",
            timestamp: timestamp,
            session_id: sessionA
        };
        client2.ws.send(`publish-json:${JSON.stringify(client2Message)}`);
        log("Client2 published NetworkConnectedEvent in session-A");
        
        // Wait a bit before sending the next batch
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Client3 publishes DetectCustomerEvent in session-B
        const client3Message = {
            publisher_name: "Client3",
            topic: detectEvent,
            payload: "Customer detected in session B",
            timestamp: timestamp,
            session_id: sessionB
        };
        client3.ws.send(`publish-json:${JSON.stringify(client3Message)}`);
        log("Client3 published DetectCustomerEvent in session-B");
        
        // Client4 publishes RegistrationCompleteEvent in session-B
        const client4Message = {
            publisher_name: "Client4",
            topic: registrationEvent,
            payload: "Registration complete from session B",
            timestamp: timestamp,
            session_id: sessionB
        };
        client4.ws.send(`publish-json:${JSON.stringify(client4Message)}`);
        log("Client4 published RegistrationCompleteEvent in session-B");
        
        // Store clients in a global variable to access them later
        window.activeClients = {
            client1, client2, client3, client4
        };
        
        // Wait to ensure all messages are processed
        log("Test running, waiting for all messages to be processed...", "info");
        
    } catch (error) {
        log(`Error in WebSocket test: ${error.message}`, "error");
        console.error("Full error:", error);
    }
}

// Helper function to setup a client
async function setupClient(clientName, sessionId, wsUrl, authUrl, username, password) {
    try {
        // Create an authenticated WebSocket connection
        const ws = await createAuthenticatedWebSocket(
            wsUrl, 
            authUrl, 
            username, 
            password, 
            sessionId
        );
        
        log(`${clientName}: Authentication successful with session ${sessionId}`, 'success');
        
        // Set up message handler with client name
        ws.onmessage = (event) => {
            try {
                // Try to parse the message as JSON
                const data = JSON.parse(event.data);
                
                // Format the client and session for display
                const clientSessionTag = `[${clientName}:${sessionId}]`;
                
                // Log received messages
                log(`${clientSessionTag} Received message: Topic=${data.topic}, Payload=${data.payload}`, 'success');
                log(`${clientSessionTag} Message details: Publisher=${data.publisher_name}, Session=${data.session_id}`, 'info');
            } catch (error) {
                // Handle non-JSON messages
                log(`${clientName} received non-JSON message: ${event.data}`, 'info');
            }
        };
        
        // Set up error handler
        ws.onerror = (error) => {
            log(`${clientName} WebSocket error: ${error}`, 'error');
        };
        
        // Set up close handler
        ws.onclose = (event) => {
            log(`${clientName} WebSocket connection closed: Code=${event.code}`, 'warning');
        };
        
        // Register client name
        ws.send(`register-name:${clientName}`);
        
        // Register session ID
        ws.send(`register-session:${sessionId}`);
        
        return { ws, name: clientName, sessionId };
    } catch (error) {
        log(`Error setting up ${clientName}: ${error.message}`, 'error');
        throw error;
    }
}

// DOM ready event listener
document.addEventListener('DOMContentLoaded', function() {
    // Check Connectivity button
    document.getElementById('checkConnectivityBtn').addEventListener('click', function() {
        checkConnectivity();
    });
    
    // Run Test button
    document.getElementById('runTestBtn').addEventListener('click', function() {
        runTest();
    });
    
    // Clear Log button
    document.getElementById('clearLogBtn').addEventListener('click', function() {
        document.getElementById('logOutput').innerHTML = '';
    });
    
    // Close Connection button
    const closeConnectionBtn = document.getElementById('closeConnectionBtn');
    if (closeConnectionBtn) {
        closeConnectionBtn.addEventListener('click', function() {
            if (window.activeClients) {
                const { client1, client2, client3, client4 } = window.activeClients;
                if (client1 && client1.ws) client1.ws.close();
                if (client2 && client2.ws) client2.ws.close();
                if (client3 && client3.ws) client3.ws.close();
                if (client4 && client4.ws) client4.ws.close();
                
                // Clear token and hide the button
                jwtManager.clearToken();
                closeConnectionBtn.style.display = 'none';
                
                log("All connections manually closed", 'info');
                
                // Clear the active clients
                window.activeClients = null;
            } else {
                log("No active connections to close", 'warning');
            }
        });
    }
    
    // Initial log message
    log("WebSocket Test Client loaded. Click a button to start.", "info");
});
