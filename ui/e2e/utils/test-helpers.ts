import * as fs from 'fs';
import * as path from 'path';
import * as net from 'net';
import { execSync, spawn } from 'child_process';
import { Page } from '@playwright/test';
import HTMXHelpers from './htmx-helpers';

interface ServerOptions {
    port?: number;
    useTestProjects?: boolean;
    timeout?: number;
    retries?: number;
}

interface ProjectSetup {
    projects: Array<{ name: string; language: string }>;
    baseUrl: string;
    projectPaths: Array<string>;
}

interface TestProjectOptions {
    language?: string;
    basePath?: string;
    template?: string;
}

/**
 * Simplified TestHelpers - HTTP API Testing Interface
 * 
 * Uses direct HTTP API calls to the console's endpoints.
 * No MCP client needed since tools are embedded in the console.
 */
class TestHelpers {
    // Project paths
    static readonly PROJECT_ROOT = path.resolve(__dirname, '..', '..', '..');
    static readonly TEST_PROJECTS_DIR = path.resolve(__dirname, '..', '.e2e-projects');
    static readonly TEST_PROJECTS_FILE = path.join(TestHelpers.TEST_PROJECTS_DIR, '.e2e-projects.json');
    static readonly FTL_BINARY = path.join(TestHelpers.PROJECT_ROOT, 'bin', 'ftl');
    static readonly SCREENSHOTS_DIR = path.join(TestHelpers.PROJECT_ROOT, 'ui', 'e2e', 'screenshots');
    
    // Console URL (set when server starts)
    private static consoleUrl = 'http://localhost:8080';
    private static serverPort = 8080;
    
    /**
     * Make an HTTP API call to the console
     */
    private static async callAPI(endpoint: string, data: any = {}): Promise<any> {
        const url = `${this.consoleUrl}/api/mcp${endpoint}`;
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        
        if (!response.ok) {
            const text = await response.text();
            throw new Error(`API call failed: ${response.status} - ${text}`);
        }
        
        return response.json();
    }
    
    /**
     * Initialize a new FTL project
     */
    static async initProject(name: string, language = 'rust', basePath?: string): Promise<void> {
        basePath = basePath || this.TEST_PROJECTS_DIR;
        await this.callAPI('/project/init', {
            project_name: name,
            language: language,
            base_path: basePath
        });
    }
    
    /**
     * Add a tool to an existing project
     */
    static async addTool(projectPath: string, toolName: string, language = 'rust'): Promise<void> {
        await this.callAPI('/project/add', {
            project_path: projectPath,
            tool_name: toolName,
            language: language
        });
    }
    
    /**
     * Build a project
     */
    static async buildProject(projectPath: string, clean = false): Promise<void> {
        await this.callAPI('/project/build', {
            project_path: projectPath,
            clean: clean
        });
    }
    
    /**
     * Start a project server
     */
    static async startProject(projectPath: string, port?: number): Promise<void> {
        const data: any = { project_path: projectPath };
        if (port) data.port = port;
        
        await this.callAPI('/project/up', data);
    }
    
    /**
     * Stop a project server
     */
    static async stopProject(projectPath: string): Promise<void> {
        await this.callAPI('/project/stop', {
            project_path: projectPath
        });
    }
    
    /**
     * Get project status
     */
    static async getProjectStatus(projectPath: string): Promise<any> {
        // Use GET for status queries
        const url = `${this.consoleUrl}/api/mcp/project/status?path=${encodeURIComponent(projectPath)}`;
        const response = await fetch(url);
        
        if (!response.ok) {
            throw new Error(`Status check failed: ${response.status}`);
        }
        
        return response.json();
    }
    
    /**
     * Health check
     */
    static async healthCheck(): Promise<boolean> {
        try {
            const response = await fetch(`${this.consoleUrl}/api/mcp/health`);
            return response.ok;
        } catch {
            return false;
        }
    }
    
    /**
     * Clean up test projects
     */
    static cleanupTestProjects(): void {
        if (fs.existsSync(this.TEST_PROJECTS_DIR)) {
            execSync(`rm -rf ${this.TEST_PROJECTS_DIR}`, { stdio: 'ignore' });
        }
    }
    
    /**
     * Setup test projects directory
     */
    static setupTestProjectsDir(): void {
        if (!fs.existsSync(this.TEST_PROJECTS_DIR)) {
            fs.mkdirSync(this.TEST_PROJECTS_DIR, { recursive: true });
        }
    }
    
    /**
     * Generate unique project name
     */
    static generateProjectName(prefix = 'test'): string {
        const timestamp = Date.now();
        const random = Math.random().toString(36).substring(2, 8);
        return `${prefix}-${timestamp}-${random}`;
    }
    
    /**
     * Setup a single test project
     */
    static async setupSingleProject(language = 'rust'): Promise<ProjectSetup> {
        this.setupTestProjectsDir();
        
        const projectName = this.generateProjectName('single');
        const projectPath = path.join(this.TEST_PROJECTS_DIR, projectName);
        
        await this.initProject(projectName, language);
        
        return {
            projects: [{ name: projectName, language }],
            baseUrl: this.consoleUrl,
            projectPaths: [projectPath]
        };
    }
    
    /**
     * Setup multiple test projects
     */
    static async setupMultipleProjects(count = 3): Promise<ProjectSetup> {
        this.setupTestProjectsDir();
        
        const projects = [];
        const projectPaths = [];
        const languages = ['rust', 'python', 'go'];
        
        for (let i = 0; i < count; i++) {
            const projectName = this.generateProjectName(`multi-${i}`);
            const language = languages[i % languages.length];
            const projectPath = path.join(this.TEST_PROJECTS_DIR, projectName);
            
            await this.initProject(projectName, language);
            
            projects.push({ name: projectName, language });
            projectPaths.push(projectPath);
        }
        
        return {
            projects,
            baseUrl: this.consoleUrl,
            projectPaths
        };
    }
    
    /**
     * Wait for HTMX operations to settle
     */
    static async waitForHTMX(page: Page, timeout = 5000): Promise<void> {
        await HTMXHelpers.waitForSettle(page, { timeout });
    }
    
    /**
     * Wait for process status change using test endpoint
     */
    static async waitForStatus(expectedStatus: string, timeout = 30000): Promise<void> {
        const url = `${this.consoleUrl}/api/test/wait-for-status`;
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                expected_status: expectedStatus,
                timeout_ms: timeout
            })
        });
        
        if (!response.ok) {
            throw new Error(`Wait for status failed: ${response.status}`);
        }
    }
    
    /**
     * Wait for logs to appear using test endpoint
     */
    static async waitForLogs(minLines = 10, timeout = 30000): Promise<void> {
        const url = `${this.consoleUrl}/api/test/wait-for-logs`;
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                min_lines: minLines,
                timeout_ms: timeout
            })
        });
        
        if (!response.ok) {
            throw new Error(`Wait for logs failed: ${response.status}`);
        }
    }
    
    /**
     * Get process tree for debugging
     */
    static async getProcessTree(): Promise<any> {
        const url = `${this.consoleUrl}/api/test/process-tree`;
        const response = await fetch(url);
        
        if (!response.ok) {
            throw new Error(`Get process tree failed: ${response.status}`);
        }
        
        return response.json();
    }
    
    /**
     * Find an available port starting from the given port
     */
    static async findAvailablePort(startPort = 8080): Promise<number> {
        const isPortAvailable = (port: number): Promise<boolean> => {
            return new Promise((resolve) => {
                const server = net.createServer();
                server.once('error', () => resolve(false));
                server.once('listening', () => {
                    server.close();
                    resolve(true);
                });
                server.listen(port, '127.0.0.1');
            });
        };
        
        let port = startPort;
        while (port < startPort + 100) { // Try up to 100 ports
            if (await isPortAvailable(port)) {
                return port;
            }
            port++;
        }
        throw new Error(`No available ports found starting from ${startPort}`);
    }
    
    /**
     * Start console server (for test setup)
     */
    static async startConsoleServer(options: ServerOptions = {}): Promise<void> {
        const port = options.port || 8080;
        
        // Update our static URLs
        this.serverPort = port;
        this.consoleUrl = `http://localhost:${port}`;
        
        // Check if already running on this port
        if (await this.healthCheck()) {
            console.log(`Console server already running on port ${port}`);
            return;
        }
        
        // Start console in test mode (using spawn for background process)
        const env = { ...process.env, FTL_TEST_MODE: 'true' };
        const child = spawn(this.FTL_BINARY, ['dev', 'console', '--port', port.toString()], {
            env,
            stdio: 'ignore',
            detached: true
        });
        child.unref(); // Allow parent process to exit independently
        
        // Wait for server to be ready
        const maxRetries = options.retries || 30;
        for (let i = 0; i < maxRetries; i++) {
            if (await this.healthCheck()) {
                console.log(`Console server started successfully on port ${port}`);
                return;
            }
            await new Promise(resolve => setTimeout(resolve, 1000));
        }
        
        throw new Error(`Console server failed to start on port ${port}`);
    }
    
    /**
     * Stop console server (for test cleanup)
     */
    static stopConsoleServer(): void {
        try {
            execSync('pkill -f "ftl dev console"', { stdio: 'ignore' });
        } catch {
            // Ignore errors if process not found
        }
    }
}

export default TestHelpers;
export { TestHelpers, ProjectSetup, TestProjectOptions, ServerOptions };