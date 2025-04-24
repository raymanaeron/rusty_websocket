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
async function createClient(clientName, wsUrl, topics, publishAction) {
    log(`[connect] client_name=${clientName}, ws_url=${wsUrl} -- executing`);

    const ws = new WebSocket(wsUrl);

    // Handle incoming messages from the WebSocket server
    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            const topic = data.topic || "<unknown>";
            const payload = data.payload || "<no payload>";
            const publisher = data.publisher_name || "<unknown>";
            const timestamp = data.timestamp || "???";

            log(`[on_message] ${clientName} <- topic=${topic}, payload=${payload}, publisher=${publisher}, timestamp=${timestamp}`);
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

            // Subscribe to the specified topics
            for (const topic of topics.subscriptions) {
                const payload = "web-client-subscription";
                log(`[subscribe] subscriber_name=${clientName}, topic=${topic}, payload=${payload}`);
                ws.send(`subscribe:${topic}`);
            }

            // Publish a message to a specific topic after a short delay
            setTimeout(() => {
                const message = {
                    publisher_name: clientName,
                    topic: publishAction.topic,
                    payload: publishAction.message,
                    timestamp: getTimestamp()
                };
            
                try {
                    ws.send(`publish-json:${JSON.stringify(message)}`);
                    log(`[publish] ${clientName} sent to ${message.topic} with payload=${message.payload}`);
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
    await Promise.all([
        createClient("Client1", "ws://localhost:8081/ws",
            { subscriptions: [DetectCustomerEvent, NetworkConnectedEvent] },
            { topic: RegistrationCompleteEvent, message: "Registration complete" }
        ),
        createClient("Client2", "ws://localhost:8081/ws",
            { subscriptions: [DetectCustomerEvent, RegistrationCompleteEvent] },
            { topic: NetworkConnectedEvent, message: "Network connected" }
        ),
        createClient("Client3", "ws://localhost:8081/ws",
            { subscriptions: [NetworkConnectedEvent, RegistrationCompleteEvent] },
            { topic: DetectCustomerEvent, message: "Customer detected" }
        )
    ]);

    // Log a message indicating that all tests are complete
    log("[TestRunner] All clients launched and tests completed.");
}

// Attach the test runner to the "Start" button in the UI
document.getElementById("startBtn").addEventListener("click", runTest);
