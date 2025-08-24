const { spawn } = require('child_process');
const net = require('net');
const fs = require('fs');
const path = require('path');
const TestLifecycle = require('./TestLifecycle');

class ServerManager {
    constructor() {
        this.serverProcess = null;
        this.port = null;
        this.baseURL = null;
    }

    /**
     * Find an available port dynamically using TestLifecycle
     */
    async findAvailablePort(startPort = 8080) {
        // Use TestLifecycle's MCP-based port finder
        const port = await TestLifecycle.findAvailablePort(startPort, startPort + 100);
        if (port) {
            return port;
        }
        throw new Error('No available ports found');
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
     * Start the FTL dev console server with dynamic port
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
        
        console.log(`Starting FTL dev console on dynamic port ${this.port}...`);

        // Set environment for test projects if needed
        const env = { ...process.env };
        
        // Enable test mode for access to test endpoints
        env.FTL_TEST_MODE = 'true';
        
        if (useTestProjects) {
            const testProjectsFile = path.join(process.cwd(), '.e2e-projects.json');
            env.PROJECTS_FILE = testProjectsFile;
            
            // Ensure test projects file exists
            if (!fs.existsSync(testProjectsFile)) {
                fs.writeFileSync(testProjectsFile, '[]');
            }
        }

        // Try to start server with retries
        for (let attempt = 1; attempt <= retries; attempt++) {
            try {
                await this.attemptServerStart(env, timeout);
                console.log(`✅ Server started successfully on port ${this.port}`);
                return this.baseURL;
            } catch (error) {
                console.log(`❌ Attempt ${attempt} failed: ${error.message}`);
                if (attempt === retries) {
                    throw new Error(`Failed to start server after ${retries} attempts`);
                }
                // Wait before retry
                await new Promise(resolve => setTimeout(resolve, 2000));
            }
        }
    }

    /**
     * Attempt to start the server
     */
    attemptServerStart(env, timeout) {
        return new Promise((resolve, reject) => {
            // Kill any existing process first
            if (this.serverProcess) {
                this.killServer();
            }

            // Start new server process  
            // Use absolute path to the built binary
            const ftlPath = path.join(process.cwd(), 'bin', 'ftl');
            this.serverProcess = spawn(ftlPath, ['dev', 'console', '--port', this.port.toString()], {
                env,
                cwd: process.cwd(),
                detached: false
            });

            let serverReady = false;
            const timeoutHandle = setTimeout(() => {
                if (!serverReady) {
                    this.killServer();
                    reject(new Error('Server startup timeout'));
                }
            }, timeout);

            // Monitor stdout for ready signal
            this.serverProcess.stdout.on('data', (data) => {
                const output = data.toString();
                console.log(`[Server]: ${output}`);
                
                // Check for server ready indicators
                if (output.includes('Server running') || 
                    output.includes('Listening on') ||
                    output.includes(`${this.port}`)) {
                    serverReady = true;
                    clearTimeout(timeoutHandle);
                    
                    // Give server a moment to fully initialize
                    setTimeout(() => resolve(), 1000);
                }
            });

            this.serverProcess.stderr.on('data', (data) => {
                console.error(`[Server Error]: ${data.toString()}`);
            });

            this.serverProcess.on('error', (error) => {
                clearTimeout(timeoutHandle);
                reject(new Error(`Failed to start server: ${error.message}`));
            });

            this.serverProcess.on('exit', (code, signal) => {
                if (!serverReady) {
                    clearTimeout(timeoutHandle);
                    reject(new Error(`Server exited unexpectedly with code ${code}, signal ${signal}`));
                }
            });

            // Also use TestLifecycle to check readiness after initial delay
            setTimeout(async () => {
                if (!serverReady) {
                    try {
                        // Use TestLifecycle's waitForReady with MCP tools
                        const projectPath = env.FTL_PROJECT_PATH || '/tmp/test-project';
                        const isReady = await TestLifecycle.waitForReady(projectPath, {
                            timeout: 5000,
                            maxAttempts: 3,
                            mcpServerPath: './mcp-server'
                        });
                        
                        if (isReady) {
                            serverReady = true;
                            clearTimeout(timeoutHandle);
                            resolve();
                        }
                    } catch (error) {
                        // Server not ready yet
                    }
                }
            }, 3000);
        });
    }

    /**
     * Kill the server and clean up using MCP tools
     */
    async killServer() {
        if (this.serverProcess) {
            const pid = this.serverProcess.pid;
            console.log(`Killing server process (PID: ${pid}) on port ${this.port}...`);
            
            try {
                // Use MCP kill_gracefully tool for cross-platform support
                const killResult = await TestLifecycle.killProcessGracefully(pid, { timeout: 3 });
                
                if (killResult.success) {
                    console.log(`✅ Server killed successfully using ${killResult.method}`);
                } else {
                    console.warn(`⚠️ Failed to kill server: ${killResult.message}`);
                }
            } catch (error) {
                // Fallback to Node.js kill if MCP tools unavailable
                console.log('Falling back to Node.js process kill');
                this.serverProcess.kill('SIGTERM');
                
                // Force kill after delay if still running
                setTimeout(() => {
                    if (this.serverProcess && !this.serverProcess.killed) {
                        this.serverProcess.kill('SIGKILL');
                    }
                }, 2000);
            }

            this.serverProcess = null;
        }

        // Clean up any orphaned processes on the port
        await this.cleanupPort();
    }

    /**
     * Clean up any processes using our port using MCP tools
     */
    async cleanupPort() {
        if (!this.port) return;

        try {
            // Use MCP cleanup_orphans tool for cross-platform port cleanup
            const cleanupResult = await TestLifecycle.cleanupOrphans({
                port: this.port,
                killOrphans: true
            });
            
            if (cleanupResult.killedCount > 0) {
                console.log(`✅ Cleaned up ${cleanupResult.killedCount} processes on port ${this.port}`);
            }
        } catch (error) {
            // Fallback to platform-specific cleanup
            try {
                const { exec } = require('child_process');
                exec(`lsof -ti:${this.port} | xargs kill -9`, (error) => {
                    if (!error) {
                        console.log(`Cleaned up processes on port ${this.port}`);
                    }
                });
            } catch (error) {
                // Ignore errors in cleanup
            }
        }
    }

    /**
     * Clean up orphaned browser processes using MCP tools
     */
    static async cleanupOrphanedBrowsers() {
        try {
            // Use MCP cleanup_orphans tool for cross-platform browser cleanup
            const chromiumResult = await TestLifecycle.cleanupOrphans({
                processName: 'chromium',
                killOrphans: true
            });
            
            const chromeResult = await TestLifecycle.cleanupOrphans({
                processName: 'chrome',
                killOrphans: true
            });
            
            const totalKilled = chromiumResult.killedCount + chromeResult.killedCount;
            if (totalKilled > 0) {
                console.log(`✅ Cleaned up ${totalKilled} orphaned browser processes`);
            }
        } catch (error) {
            // Fallback to platform-specific cleanup
            try {
                const { execSync } = require('child_process');
                execSync('pkill -f "chromium.*--test-type=webdriver" || true', { stdio: 'ignore' });
                execSync('pkill -f "chrome.*--headless" || true', { stdio: 'ignore' });
                console.log('Cleaned up orphaned browser processes');
            } catch (error) {
                // Ignore errors
            }
        }
    }

    /**
     * Get the current server URL
     */
    getBaseURL() {
        if (!this.baseURL) {
            throw new Error('Server not started');
        }
        return this.baseURL;
    }

    /**
     * Get the current port
     */
    getPort() {
        return this.port;
    }
}

module.exports = ServerManager;