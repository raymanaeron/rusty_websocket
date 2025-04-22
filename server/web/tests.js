function log(msg) {
    const logBox = document.getElementById("log");
    logBox.textContent += msg + "\n";
    logBox.scrollTop = logBox.scrollHeight;
  }
  
  async function createClient(name, topics, publishAction) {
    const ws = new WebSocket("ws://localhost:8081/ws");
  
    ws.onmessage = (event) => {
      log(`[${name}] Received: ${event.data}`);
    };
  
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
  
  document.getElementById("startBtn").addEventListener("click", runTest);
  