import { spawn, execSync, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as net from 'net';
import TestHelpers from './test-helpers';

interface ServerOptions {
    useTestProjects?: boolean;
    timeout?: number;
    retries?: number;
}

class ServerManager {
    private serverProcess: ChildProcess | null = null;
    private port: number | null = null;
    private baseURL: string | null = null;

    /**
     * Find an available port
     */
    async findAvailablePort(startPort: number = 8080, maxAttempts: number = 10): Promise<number> {
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
    isPortAvailable(port: number): Promise<boolean> {
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
    async startServer(options: ServerOptions = {}): Promise<string> {
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
            env.PROJECTS_FILE = TestHelpers.TEST_PROJECTS_FILE;
        }
        env.FTL_TEST_MODE = 'true';

        // Create test projects file in the correct location
        if (useTestProjects) {
            // Ensure directory exists
            if (!fs.existsSync(TestHelpers.TEST_PROJECTS_DIR)) {
                fs.mkdirSync(TestHelpers.TEST_PROJECTS_DIR, { recursive: true });
            }
            
            // Create file if it doesn't exist
            if (!fs.existsSync(TestHelpers.TEST_PROJECTS_FILE)) {
                fs.writeFileSync(TestHelpers.TEST_PROJECTS_FILE, '[]');
            }
        }

        console.log(`Starting server on port ${this.port}...`);
        console.log(`Using FTL binary: ${TestHelpers.FTL_BINARY}`);

        // Verify FTL binary exists and provide helpful error message
        TestHelpers.verifyFtlBinaryExists();

        // Start server with proper path resolution
        this.serverProcess = spawn(TestHelpers.FTL_BINARY, ['dev', 'console', '--port', this.port.toString()], {
            cwd: TestHelpers.PROJECT_ROOT,
            env,
            stdio: ['ignore', 'pipe', 'pipe'],
            detached: false
        });

        // Handle spawn errors with clear messaging
        this.serverProcess.on('error', (error) => {
            console.error(`Server spawn error: ${error.message}`);
            throw new Error(`Failed to start FTL server: ${error.message}`);
        });

        // Wait for server to be ready
        await this.waitForServer(timeout, retries);

        return this.baseURL;
    }

    /**
     * Wait for server to be ready
     */
    async waitForServer(timeout: number = 30000, retries: number = 3): Promise<boolean> {
        const startTime = Date.now();
        
        while (Date.now() - startTime < timeout) {
            try {
                const response = await fetch(this.baseURL!);
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
    killServer(): void {
        if (this.serverProcess) {
            console.log('Stopping server...');
            this.serverProcess.kill('SIGTERM');
            this.serverProcess = null;
        }
    }

    /**
     * Get the server port
     */
    getPort(): number | null {
        return this.port;
    }

    /**
     * Get the base URL
     */
    getBaseURL(): string | null {
        return this.baseURL;
    }

    /**
     * Clean up orphaned browser processes
     */
    static cleanupOrphanedBrowsers(): void {
        if (process.platform === 'darwin' || process.platform === 'linux') {
            execSync('pkill -f chromium || true', { stdio: 'ignore' });
            execSync('pkill -f playwright || true', { stdio: 'ignore' });
        }
    }
}

export default ServerManager;