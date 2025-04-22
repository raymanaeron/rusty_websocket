// Logs a message to the log box in the UI
function log(msg) {
    const logBox = document.getElementById("log");
    logBox.textContent += msg + "\n";
    logBox.scrollTop = logBox.scrollHeight; // Auto-scroll to the latest log
}

// Creates a WebSocket client, subscribes to topics, and publishes a message
async function createClient(name, topics, publishAction) {
    const ws = new WebSocket("ws://localhost:8081/ws");

    // Handle incoming messages from the WebSocket server
    ws.onmessage = (event) => {
        log(`[${name}] Received: ${event.data}`);
    };

    // Wait for the WebSocket connection to open
    return new Promise((resolve) => {
        ws.onopen = async () => {
            // Register the client name with the server
            ws.send(`register-name:${name}`);

            // Subscribe to the specified topics
            for (const topic of topics.subscriptions) {
                ws.send(`subscribe:${topic}`);
                log(`[${name}] subscribed to ${topic}`);
            }

            // Publish a message to a specific topic after a short delay
            setTimeout(() => {
                ws.send(`publish:${publishAction.topic}:${publishAction.message}`);
                log(`[${name}] Published to ${publishAction.topic}: ${publishAction.message}`);
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
        createClient("Client1",
            { subscriptions: [DetectCustomerEvent, NetworkConnectedEvent] },
            { topic: RegistrationCompleteEvent, message: "Registration complete" }
        ),
        createClient("Client2",
            { subscriptions: [DetectCustomerEvent, RegistrationCompleteEvent] },
            { topic: NetworkConnectedEvent, message: "Network connected" }
        ),
        createClient("Client3",
            { subscriptions: [NetworkConnectedEvent, RegistrationCompleteEvent] },
            { topic: DetectCustomerEvent, message: "Customer detected" }
        )
    ]);

    // Log a message indicating that all tests are complete
    log("[TestRunner] All clients launched and tests completed.");
}

// Attach the test runner to the "Start" button in the UI
document.getElementById("startBtn").addEventListener("click", runTest);
