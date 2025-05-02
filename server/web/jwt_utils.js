// Web client JWT authentication utilities

/**
 * Handles JWT token authentication and management
 */
class JwtManager {
    constructor() {
        this.token = null;
        this.expiresAt = null;
        this.refreshTimerId = null;
    }

    /**
     * Authenticates with the server and gets a JWT token
     * @param {string} authUrl - The authentication API URL
     * @param {string} username - Username for authentication
     * @param {string} password - Password for authentication
     * @param {string|null} sessionId - Optional session ID
     * @returns {Promise<string>} - The JWT token
     */
    async authenticate(authUrl, username, password, sessionId = null) {
        try {
            // Prepare the authentication request
            const authRequest = {
                username,
                password
            };

            // Add session ID if provided
            if (sessionId) {
                authRequest.session_id = sessionId;
            }

            console.log(`Auth request to ${authUrl}:`, authRequest);

            // Make the authentication request
            const response = await fetch(authUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(authRequest)
            });

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Authentication failed with status: ${response.status}`, errorText);
                throw new Error(`Authentication failed with status: ${response.status}`);
            }

            const authData = await response.json();
            console.log("Authentication response:", authData);
            this.token = authData.token;
            
            // Log token details (first 20 chars only for security)
            console.log(`Token received (first 20 chars): ${this.token.substring(0, 20)}...`);
            
            // Calculate expiration time (subtract 5 minutes for safety margin)
            const expiresInMs = (authData.expires_in - 300) * 1000;
            this.expiresAt = Date.now() + expiresInMs;
            console.log(`Token expires at: ${new Date(this.expiresAt).toLocaleString()}`);
            
            // Setup token refresh timer
            this.setupRefreshTimer(authUrl, username, password, sessionId, expiresInMs);
            
            return this.token;
        } catch (error) {
            console.error('JWT authentication error:', error);
            throw error;
        }
    }

    /**
     * Sets up a timer to refresh the token before it expires
     * @param {string} authUrl - The authentication API URL
     * @param {string} username - Username for authentication
     * @param {string} password - Password for authentication
     * @param {string|null} sessionId - Optional session ID
     * @param {number} refreshTimeMs - When to refresh the token (in milliseconds)
     */
    setupRefreshTimer(authUrl, username, password, sessionId, refreshTimeMs) {
        // Clear any existing timer
        if (this.refreshTimerId) {
            clearTimeout(this.refreshTimerId);
        }
        
        // Set timer to refresh token before it expires
        this.refreshTimerId = setTimeout(async () => {
            console.log("Token is about to expire, refreshing...");
            try {
                await this.authenticate(authUrl, username, password, sessionId);
                console.log("Token refreshed successfully");
            } catch (error) {
                console.error("Failed to refresh token:", error);
            }
        }, refreshTimeMs);
    }

    /**
     * Gets the current token if it's still valid
     * @returns {string|null} - The JWT token or null if no valid token
     */
    getToken() {
        if (!this.token || !this.expiresAt) {
            return null;
        }
        
        // Return the token only if it's still valid
        if (Date.now() < this.expiresAt) {
            return this.token;
        }
        
        return null;
    }

    /**
     * Checks if the client is authenticated with a valid token
     * @returns {boolean} - True if client has a valid token
     */
    isAuthenticated() {
        return this.getToken() !== null;
    }

    /**
     * Clears the current token and cancels any refresh timer
     */
    clearToken() {
        this.token = null;
        this.expiresAt = null;
        
        if (this.refreshTimerId) {
            clearTimeout(this.refreshTimerId);
            this.refreshTimerId = null;
        }
    }
}

// Create a singleton instance for the application
const jwtManager = new JwtManager();

/**
 * Creates an authenticated WebSocket connection
 * @param {string} wsUrl - WebSocket URL to connect to
 * @param {string} authUrl - Authentication API URL
 * @param {string} username - Username for authentication
 * @param {string} password - Password for authentication
 * @param {string|null} sessionId - Optional session ID
 * @returns {Promise<WebSocket>} - WebSocket connection with authentication
 */
async function createAuthenticatedWebSocket(wsUrl, authUrl, username, password, sessionId = null) {
    try {
        // Authenticate and get token
        const token = await jwtManager.authenticate(authUrl, username, password, sessionId);
        
        // Add token to WebSocket URL as query parameter
        const wsUrlWithToken = new URL(wsUrl);
        wsUrlWithToken.searchParams.append('token', token);
        
        console.log(`Connecting to WebSocket with token (showing first 20 chars): ${token.substring(0, 20)}...`);
        console.log(`Full WebSocket URL: ${wsUrlWithToken.toString()}`);
        
        // Create WebSocket connection with token
        const ws = new WebSocket(wsUrlWithToken.toString());
        
        // Return promise that resolves when connection is established
        return new Promise((resolve, reject) => {
            ws.onopen = () => {
                console.log('Authenticated WebSocket connection established successfully');
                resolve(ws);
            };
            
            ws.onerror = (error) => {
                console.error('WebSocket connection error:', error);
                reject(error);
            };
            
            // Add a timeout to reject if connection isn't established within 5 seconds
            setTimeout(() => {
                if (ws.readyState !== WebSocket.OPEN) {
                    console.error('WebSocket connection timed out');
                    reject(new Error('WebSocket connection timed out after 5 seconds'));
                }
            }, 5000);
        });
    } catch (error) {
        console.error('Failed to create authenticated WebSocket:', error);
        throw error;
    }
}

// Export the JWT utilities
export {
    jwtManager,
    createAuthenticatedWebSocket
};
