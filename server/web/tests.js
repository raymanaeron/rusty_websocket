// Logs a message to the log box in the UI
function log(msg) {
    const logBox = document.getElementById("log");
    logBox.textContent += msg + "\n";
    logBox.scrollTop = logBox.scrollHeight;
}

// Creates a WebSocket client, subscribes to topics, and publishes a message
async function createClient(name, topics, publishAction) {
    const ws = new WebSocket("ws://localhost:8081/ws");

    // Handle incoming JSON messages from the WebSocket server
    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            const topic = data.topic || "<unknown>";
            const message = data.message || "<no message>";
            log(`[${name}] [${topic}] -> ${message}`);
        } catch (err) {
            log(`[${name}] Malformed message: ${event.data}`);
        }
    };

    // Wait for the WebSocket connection to open
    return new Promise((resolve) => {
        ws.onopen = async () => {
            ws.send(`register-name:${name}`);

            for (const topic of topics.subscriptions) {
                ws.send(`subscribe:${topic}`);
                log(`[${name}] subscribed to ${topic}`);
            }

            setTimeout(() => {
                ws.send(`publish:${publishAction.topic}:${publishAction.message}`);
                log(`[${name}] Published to ${publishAction.topic}: ${publishAction.message}`);
            }, 500);

            resolve(ws);
        };
    });
}

// Runs the test by creating multiple clients and simulating interactions
async function runTest() {
    document.getElementById("log").textContent = "";

    const DetectCustomerEvent = "DetectCustomerEvent";
    const NetworkConnectedEvent = "NetworkConnectedEvent";
    const RegistrationCompleteEvent = "RegistrationCompleteEvent";

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

    log("[TestRunner] All clients launched and tests completed.");
}

// Attach the test runner to the "Start" button in the UI
document.getElementById("startBtn").addEventListener("click", runTest);
