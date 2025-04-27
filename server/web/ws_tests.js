// web/ws_tests.js
// WebSocket tests for the Rusty WebSocket server
// These tests assume that the server is running and accessible

// Import the necessary libraries
import test from 'ava';
import WebSocket from 'ws';

// Define the WebSocket server URL
const url = 'ws://127.0.0.1:8081/ws';

// Test the WebSocket connection
test('WebSocket connection', t => {
    const ws = new WebSocket(url);

    ws.on('open', () => {
        t.pass();
        ws.close();
    });

    ws.on('error', () => {
        t.fail('WebSocket connection error');
    });
});

// Test sending and receiving a message
test('WebSocket send and receive message', t => {
    const ws = new WebSocket(url);

    const message = 'Hello, WebSocket!';
    ws.on('open', () => {
        ws.send(message);
    });

    ws.on('message', (receivedMessage) => {
        t.is(receivedMessage, message);
        ws.close();
    });

    ws.on('error', () => {
        t.fail('WebSocket connection error');
    });
});