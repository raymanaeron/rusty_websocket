// Simple non-module script to test logging

// Wait for DOM to load
document.addEventListener('DOMContentLoaded', function() {
    console.log("DOM fully loaded");
    
    // Test direct manipulation of the log element
    const logOutput = document.getElementById('logOutput');
    if (logOutput) {
        console.log("Log element found");
        
        // Add a test message directly
        const testEntry = document.createElement('div');
        testEntry.textContent = "Direct test message";
        testEntry.style.color = "blue";
        logOutput.appendChild(testEntry);
    } else {
        console.error("Log element not found!");
    }
    
    // Add a button click handler to test dynamic logging
    const testLogBtn = document.createElement('button');
    testLogBtn.textContent = "Test Log";
    testLogBtn.onclick = function() {
        const logOutput = document.getElementById('logOutput');
        if (logOutput) {
            const entry = document.createElement('div');
            entry.textContent = "Button clicked at " + new Date().toLocaleTimeString();
            entry.style.color = "green";
            logOutput.appendChild(entry);
        }
    };
    
    // Add the test button to the page
    document.body.insertBefore(testLogBtn, logOutput);
});
