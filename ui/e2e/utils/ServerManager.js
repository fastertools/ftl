const { spawn, execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const net = require('net');

class ServerManager {
    constructor() {
        this.serverProcess = null;
        this.port = null;
        this.baseURL = null;
    }

    /**
     * Find an available port
     */
    async findAvailablePort(startPort = 8080, maxAttempts = 10) {
        for (let i = 0; i < maxAttempts; i++) {
            const port = startPort + i;
            const isAvailable = await this.isPortAvailable(port);
            if (isAvailable) {
                return port;
            }
        }
        throw new Error(`No available ports found in range ${startPort}-${startPort + maxAttempts}`);
    }

    /**
     * Check if a port is available
     */
    isPortAvailable(port) {
        return new Promise((resolve) => {
            const server = net.createServer();
            server.once('error', () => resolve(false));
            server.once('listening', () => {
                server.close();
                resolve(true);
            });
            server.listen(port);
        });
    }

    /**
     * Start the FTL server
     */
    async startServer(options = {}) {
        const {
            useTestProjects = true,
            timeout = 30000,
            retries = 3
        } = options;

        // Find available port
        this.port = await this.findAvailablePort();
        this.baseURL = `http://localhost:${this.port}`;

        // Setup environment
        const env = { ...process.env };
        if (useTestProjects) {
            env.PROJECTS_FILE = '.e2e-projects.json';
        }
        env.FTL_TEST_MODE = 'true';

        // Create test projects file
        if (useTestProjects && !fs.existsSync('.e2e-projects.json')) {
            fs.writeFileSync('.e2e-projects.json', '[]');
        }

        console.log(`Starting server on port ${this.port}...`);

        // Start server
        this.serverProcess = spawn('./bin/ftl', ['dev', 'console', '--port', this.port.toString()], {
            env,
            stdio: ['ignore', 'pipe', 'pipe'],
            detached: false
        });

        // Wait for server to be ready
        await this.waitForServer(timeout, retries);

        return this.baseURL;
    }

    /**
     * Wait for server to be ready
     */
    async waitForServer(timeout = 30000, retries = 3) {
        const startTime = Date.now();
        
        while (Date.now() - startTime < timeout) {
            try {
                const response = await fetch(this.baseURL);
                if (response.ok) {
                    console.log('Server is ready');
                    return true;
                }
            } catch (error) {
                // Server not ready yet
            }
            await new Promise(resolve => setTimeout(resolve, 1000));
        }
        
        throw new Error(`Server failed to start within ${timeout}ms`);
    }

    /**
     * Kill the server process
     */
    killServer() {
        if (this.serverProcess) {
            console.log('Stopping server...');
            this.serverProcess.kill('SIGTERM');
            this.serverProcess = null;
        }
    }

    /**
     * Get the server port
     */
    getPort() {
        return this.port;
    }

    /**
     * Get the base URL
     */
    getBaseURL() {
        return this.baseURL;
    }

    /**
     * Clean up orphaned browser processes
     */
    static cleanupOrphanedBrowsers() {
        try {
            if (process.platform === 'darwin' || process.platform === 'linux') {
                execSync('pkill -f chromium || true', { stdio: 'ignore' });
                execSync('pkill -f playwright || true', { stdio: 'ignore' });
            }
        } catch (error) {
            // Ignore errors
        }
    }
}

module.exports = ServerManager;