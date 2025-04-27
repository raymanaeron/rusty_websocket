// Logs a message to the log box in the UI
function log(msg) {
    const logBox = document.getElementById("log");
    logBox.textContent += msg + "\n";
    logBox.scrollTop = logBox.scrollHeight; // Auto-scroll to the latest log
}

// Returns the current timestamp in ISO format
function getTimestamp() {
    return new Date().toISOString();
}

// Creates a WebSocket client, subscribes to topics, and publishes a message
async function createClient(clientName, wsUrl, topics, publishAction, sessionId = null) {
    // If no session ID is provided, use the client name as the session
    const clientSession = sessionId || `session-${clientName}`;
    
    log(`[connect] client_name=${clientName}, session_id=${clientSession}, ws_url=${wsUrl} -- executing`);

    const ws = new WebSocket(wsUrl);

    // Handle incoming messages from the WebSocket server
    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            const topic = data.topic || "<unknown>";
            const payload = data.payload || "<no payload>";
            const publisher = data.publisher_name || "<unknown>";
            const timestamp = data.timestamp || "???";
            const msgSession = data.session_id || "<unknown>";

            log(`[on_message] ${clientName} <- topic=${topic}, payload=${payload}, publisher=${publisher}, timestamp=${timestamp}, session=${msgSession}`);
        } catch (err) {
            log(`[on_message] ${clientName} received malformed message: ${event.data}`);
        }
    };

    // Wait for the WebSocket connection to open
    return new Promise((resolve) => {
        ws.onopen = async () => {
            // Register the client name with the server
            ws.send(`register-name:${clientName}`);
            log(`[connect] ${clientName} registered.`);
            
            // Register the session ID with the server
            ws.send(`register-session:${clientSession}`);
            log(`[connect] Session ${clientSession} registered.`);

            // Subscribe to the specified topics
            for (const topic of topics.subscriptions) {
                const payload = "web-client-subscription";
                log(`[subscribe] subscriber_name=${clientName}, topic=${topic}, session=${clientSession}, payload=${payload}`);
                ws.send(`subscribe:${topic}|${clientSession}`);
            }

            // Publish a message to a specific topic after a short delay
            setTimeout(() => {
                const message = {
                    publisher_name: clientName,
                    topic: publishAction.topic,
                    payload: publishAction.message,
                    timestamp: getTimestamp(),
                    session_id: clientSession
                };
            
                try {
                    ws.send(`publish-json:${JSON.stringify(message)}`);
                    log(`[publish] ${clientName} sent to ${message.topic} with payload=${message.payload}, session=${clientSession}`);
                } catch (error) {
                    log(`[publish] ${clientName} failed to send: ${error}`);
                }
            }, 500);

            resolve(ws); // Resolve the promise once the client is set up
        };
    });
}

// Runs the test by creating multiple clients and simulating interactions
async function runTest() {
    document.getElementById("log").textContent = ""; // Clear the log box

    // Define event topics
    const DetectCustomerEvent = "DetectCustomerEvent";
    const NetworkConnectedEvent = "NetworkConnectedEvent";
    const RegistrationCompleteEvent = "RegistrationCompleteEvent";

    // Create and configure clients with subscriptions and publish actions
    // Use two different sessions to demonstrate session-based routing
    const sessionA = "session-A";
    const sessionB = "session-B";

    await Promise.all([
        // These clients are in session A
        createClient("Client1", "ws://localhost:8081/ws",
            { subscriptions: [DetectCustomerEvent, NetworkConnectedEvent] },
            { topic: RegistrationCompleteEvent, message: "Registration complete" },
            sessionA
        ),
        createClient("Client2", "ws://localhost:8081/ws",
            { subscriptions: [DetectCustomerEvent, RegistrationCompleteEvent] },
            { topic: NetworkConnectedEvent, message: "Network connected" },
            sessionA
        ),
        
        // These clients are in session B
        createClient("Client3", "ws://localhost:8081/ws",
            { subscriptions: [NetworkConnectedEvent, RegistrationCompleteEvent] },
            { topic: DetectCustomerEvent, message: "Customer detected" },
            sessionB
        ),
        createClient("Client4", "ws://localhost:8081/ws",
            { subscriptions: [DetectCustomerEvent, NetworkConnectedEvent] },
            { topic: RegistrationCompleteEvent, message: "Registration from session B" },
            sessionB
        )
    ]);

    // Log a message indicating that all tests are complete
    log("[TestRunner] All clients launched and tests completed.");
}

// Attach the test runner to the "Start" button in the UI
document.getElementById("startBtn").addEventListener("click", runTest);
