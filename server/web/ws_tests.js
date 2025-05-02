// web/ws_tests.js
// WebSocket tests for the Rusty WebSocket server
// These tests assume that the server is running and accessible

// Browser-compatible version - no Node.js imports
// Original imports removed:
// import test from 'ava';
// import WebSocket from 'ws';

// Define the WebSocket server URL
const url = 'ws://127.0.0.1:8081/ws';

// Simple test runner for browser environment
function runTest(name, testFn) {
    console.log(`Running test: ${name}`);
    try {
        testFn({
            pass: () => console.log(`✓ ${name}`),
            fail: (msg) => console.error(`✗ ${name}: ${msg}`),
            is: (a, b) => {
                if (a === b) {
                    console.log(`✓ ${name}: values match`);
                } else {
                    console.error(`✗ ${name}: expected ${b}, got ${a}`);
                }
            }
        });
    } catch (e) {
        console.error(`✗ ${name} failed with error: ${e.message}`);
    }
}

// Test the WebSocket connection
function testConnection(t) {
    const ws = new WebSocket(url);

    ws.onopen = () => {
        t.pass();
        ws.close();
    };

    ws.onerror = () => {
        t.fail('WebSocket connection error');
    };
}

// Test sending and receiving a message
function testSendReceive(t) {
    const ws = new WebSocket(url);

    const message = 'Hello, WebSocket!';
    ws.onopen = () => {
        ws.send(message);
    };

    ws.onmessage = (event) => {
        t.is(event.data, message);
        ws.close();
    };

    ws.onerror = () => {
        t.fail('WebSocket connection error');
    };
}

// Run the tests when the button is clicked
document.getElementById('startBtn').addEventListener('click', function() {
    const log = document.getElementById('log');
    
    // Clear the log
    log.innerHTML = '';
    
    // Log to both console and UI
    const originalConsoleLog = console.log;
    const originalConsoleError = console.error;
    
    console.log = function(message) {
        originalConsoleLog.apply(console, arguments);
        const div = document.createElement('div');
        div.textContent = message;
        log.appendChild(div);
    };
    
    console.error = function(message) {
        originalConsoleError.apply(console, arguments);
        const div = document.createElement('div');
        div.textContent = message;
        div.style.color = 'red';
        log.appendChild(div);
    };
    
    console.log('Starting WebSocket tests...');
    
    // Run the tests
    runTest('WebSocket connection', testConnection);
    setTimeout(() => {
        runTest('WebSocket send and receive', testSendReceive);
    }, 1000); // Run second test after a short delay
    
    // Restore console
    setTimeout(() => {
        console.log = originalConsoleLog;
        console.error = originalConsoleError;
    }, 5000);
});