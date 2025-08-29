import * as fs from 'fs';
import * as path from 'path';
import { spawn, execSync, ChildProcess } from 'child_process';
import { Page } from '@playwright/test';
import HTMXHelpers from './htmx-helpers';
import ServerManager from './server-manager';
import { MCPClient } from './mcp-client';
import { validateMCPToolInput } from './mcp-tool-validator';

interface ServerOptions {
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
 * Enhanced TestHelpers - MCP-Powered Testing Interface
 * 
 * Unified testing interface that:
 * - Uses MCP tools internally for reliability and consistency
 * - Delegates to existing HTMXHelpers for UI synchronization  
 * - Validates inputs against CUE schemas
 * - Mirrors FTL CLI patterns in testing operations
 * - Contains everything in .e2e-projects for simple cleanup
 */
class TestHelpers {
    // Project paths
    static readonly PROJECT_ROOT = path.resolve(__dirname, '..', '..', '..');
    static readonly TEST_PROJECTS_DIR = path.resolve(__dirname, '..', '.e2e-projects');
    static readonly TEST_PROJECTS_FILE = path.join(TestHelpers.TEST_PROJECTS_DIR, '.e2e-projects.json');
    static readonly FTL_BINARY = path.join(TestHelpers.PROJECT_ROOT, 'bin', 'ftl');
    static readonly SCREENSHOTS_DIR = path.join(TestHelpers.PROJECT_ROOT, 'ui', 'e2e', 'screenshots');
    static readonly TEST_CONFIG_FILE = path.join(__dirname, '.test-config.json');
    
    // MCP client instance configured for testing (lazy initialization)
    private static _mcpClient: MCPClient | null = null;
    
    private static get mcpClient(): MCPClient {
        if (!TestHelpers._mcpClient) {
            TestHelpers._mcpClient = new MCPClient({
                ftlBinaryPath: TestHelpers.FTL_BINARY,
                testMode: true,
                validateInputs: true
            });
        }
        return TestHelpers._mcpClient;
    }

    /**
     * Verify that the FTL binary exists and MCP server is available
     */
    static async verifyFtlBinaryExists(): Promise<void> {
        if (!fs.existsSync(TestHelpers.FTL_BINARY)) {
            throw new Error(
                `FTL binary not found at ${TestHelpers.FTL_BINARY}. ` +
                `Please build FTL first by running 'make build' from ${TestHelpers.PROJECT_ROOT}`
            );
        }
        
        // Verify MCP server is functional
        const isHealthy = await TestHelpers.mcpClient.checkHealth();
        if (!isHealthy) {
            console.warn('MCP server health check failed - some operations may not work properly');
        }
    }
    /**
     * Initialize test environment (replaces TestDataManager)
     */
    static async initializeTestEnvironment(createEmpty: boolean = false): Promise<void> {
        // Ensure test projects directory exists
        if (!fs.existsSync(TestHelpers.TEST_PROJECTS_DIR)) {
            fs.mkdirSync(TestHelpers.TEST_PROJECTS_DIR, { recursive: true });
        }
        
        // Clean up any existing test projects
        await TestHelpers.cleanup();
        
        if (!createEmpty) {
            // Initialize with a default test project for normal tests
            const defaultProject = await TestHelpers.initProject('default-test-project');
            console.log(`Test environment initialized with default project at: ${defaultProject}`);
        } else {
            console.log('Test environment initialized (empty)');
        }
    }

    // === Server Management (mirrors FTL CLI patterns) ===
    
    /**
     * Start development server using MCP tools
     */
    static async startServer(options: ServerOptions = {}): Promise<string> {
        try {
            // Use test-specific MCP server start tool
            const result = await TestHelpers.mcpClient.callTool('mcp-server__server_start', {
                use_test_projects: options.useTestProjects !== false,
                project_base: TestHelpers.TEST_PROJECTS_DIR,
                timeout_ms: options.timeout || 30000,
                ftl_binary_path: TestHelpers.FTL_BINARY
            });
            
            if (!result.success) {
                throw new Error(`Server start failed: ${result.error}`);
            }
            
            return result.data?.base_url || 'http://localhost:8080';
        } catch (error) {
            // Fallback to existing ServerManager for compatibility
            console.warn('MCP server start failed, falling back to ServerManager:', error);
            const serverManager = new ServerManager();
            return await serverManager.startServer(options);
        }
    }

    /**
     * Stop development server using MCP tools
     */
    static async stopServer(serverProcess?: ChildProcess): Promise<void> {
        try {
            const result = await TestHelpers.mcpClient.callTool('mcp-server__server_stop', {
                graceful: true,
                timeout_ms: 5000
            });
            
            if (!result.success) {
                console.warn('MCP server stop failed:', result.error);
            }
        } catch (error) {
            console.warn('MCP server stop error:', error);
        }
        
        // Also handle legacy serverProcess parameter
        if (serverProcess) {
            serverProcess.kill('SIGTERM');
        }
    }

    // === Project Management (mirrors FTL CLI patterns) ===
    
    /**
     * Generate unique test project name using MCP tools
     */
    static async generateTestProjectName(prefix: string = 'test'): Promise<string> {
        try {
            const result = await TestHelpers.mcpClient.callTool('mcp-server__generate_test_project_name', {
                prefix: prefix
            });
            
            if (result.success && result.data?.project_name) {
                return result.data.project_name;
            }
        } catch (error) {
            console.warn('MCP project name generation failed:', error);
        }
        
        // Fallback to simple timestamp-based generation
        return `${prefix}-${Date.now()}`;
    }

    /**
     * Generate test project path
     */
    static generateTestProjectPath(name: string): string {
        return path.join(TestHelpers.TEST_PROJECTS_DIR, name.toLowerCase().replace(/\s+/g, '_'));
    }
    
    /**
     * Initialize new FTL project using MCP tools (mirrors 'ftl init')
     */
    static async initProject(
        projectName: string, 
        options: TestProjectOptions = {}
    ): Promise<string> {
        const language = options.language || 'rust';
        const basePath = options.basePath || TestHelpers.TEST_PROJECTS_DIR;
        
        const result = await TestHelpers.mcpClient.callTool('mcp-server__init', {
            project_name: projectName,
            language,
            base_path: basePath
        });
        
        if (!result.success) {
            throw new Error(`Project initialization failed: ${result.error}`);
        }
        
        const projectPath = path.join(basePath, projectName);
        console.log(`Initialized project: ${projectName} at ${projectPath}`);
        return projectPath;
    }
    
    /**
     * Add tool to FTL project using MCP tools (mirrors 'ftl add')
     */
    static async addTool(
        projectPath: string,
        toolName: string = 'hello-world',
        language: string = 'rust'
    ): Promise<void> {
        const result = await TestHelpers.mcpClient.callTool('mcp-server__add', {
            tool_name: toolName,
            language,
            project_path: projectPath
        });
        
        if (!result.success) {
            throw new Error(`Tool addition failed: ${result.error}`);
        }
        
        console.log(`Added tool: ${toolName} to project at ${projectPath}`);
    }
    
    /**
     * Build FTL project using MCP tools (mirrors 'ftl build')
     */
    static async buildProject(projectPath: string): Promise<void> {
        const result = await TestHelpers.mcpClient.callTool('mcp-server__build', {
            project_path: projectPath
        });
        
        if (!result.success) {
            throw new Error(`Project build failed: ${result.error}`);
        }
        
        console.log(`Built project at ${projectPath}`);
    }
    
    /**
     * Start FTL project server using MCP tools (mirrors 'ftl up')
     */
    static async upProject(projectPath: string, port: number = 8080): Promise<void> {
        const result = await TestHelpers.mcpClient.callTool('mcp-server__up', {
            project_path: projectPath,
            port
        });
        
        if (!result.success) {
            throw new Error(`Project up failed: ${result.error}`);
        }
        
        console.log(`Started server for project at ${projectPath} on port ${port}`);
    }
    
    /**
     * Stop FTL project server using MCP tools (mirrors 'ftl stop')
     */
    static async stopProject(projectPath: string): Promise<void> {
        const result = await TestHelpers.mcpClient.callTool('mcp-server__stop', {
            project_path: projectPath
        });
        
        if (!result.success) {
            throw new Error(`Project stop failed: ${result.error}`);
        }
        
        console.log(`Stopped server for project at ${projectPath}`);
    }

    /**
     * Ensure test directory exists using MCP tools
     */
    static async ensureTestDirectory(testPath: string): Promise<void> {
        try {
            const result = await TestHelpers.mcpClient.callTool('mcp-server__ensure_test_directory', {
                base_path: path.dirname(testPath),
                project_name: path.basename(testPath)
            });
            
            if (!result.success) {
                throw new Error(`Test directory creation failed: ${result.error}`);
            }
        } catch (error) {
            // Fallback to manual creation for compatibility
            console.warn('MCP directory creation failed, using fallback:', error);
            
            if (!fs.existsSync(testPath)) {
                fs.mkdirSync(testPath, { recursive: true });
                
                // Initialize as a valid FTL project
                const projectName = path.basename(testPath);
                await TestHelpers.initProject(projectName, { 
                    basePath: path.dirname(testPath) 
                });
                await TestHelpers.addTool(testPath, `${projectName}-tool`);
            }
        }
    }

    // === Utility Methods ===
    
    /**
     * Take screenshot for debugging/reporting
     */
    static async takeScreenshot(page: Page, name: string): Promise<void> {
        // Ensure screenshots directory exists
        if (!fs.existsSync(TestHelpers.SCREENSHOTS_DIR)) {
            fs.mkdirSync(TestHelpers.SCREENSHOTS_DIR, { recursive: true });
        }
        
        await page.screenshot({ 
            path: path.join(TestHelpers.SCREENSHOTS_DIR, `${name}.png`),
            fullPage: true 
        });
    }
    
    /**
     * Validate MCP tool input (utility for testing)
     */
    static async validateToolInput(toolName: string, input: any): Promise<boolean> {
        try {
            const result = await validateMCPToolInput(toolName, input);
            if (!result.valid) {
                console.warn(`Tool input validation failed for ${toolName}:`, result.errors);
            }
            return result.valid;
        } catch (error) {
            console.warn('Tool input validation error:', error);
            return false;
        }
    }
    
    /**
     * Call MCP tool directly (for advanced testing scenarios)
     */
    static async callMCPTool(toolName: string, input: any = {}): Promise<any> {
        const result = await TestHelpers.mcpClient.callTool(toolName, input);
        if (!result.success) {
            throw new Error(`MCP tool call failed: ${result.error}`);
        }
        return result.data;
    }

    // === UI Interaction Helpers (delegate to existing HTMXHelpers) ===
    
    /**
     * Wait for HTMX requests to complete (delegates to existing HTMXHelpers)
     */
    static async waitForHTMX(page: Page, options: any = {}): Promise<boolean> {
        // Use existing sophisticated implementation from HTMXHelpers
        return await HTMXHelpers.waitForSettle(page, options);
    }
    
    /**
     * Wait for specific status using existing test API endpoints
     */
    static async waitForStatus(
        page: Page, 
        expectedStatus: string, 
        options: any = {}
    ): Promise<boolean> {
        // Uses existing /api/test/wait-for-status endpoint
        const result = await HTMXHelpers.waitForStatus(page, expectedStatus, options);
        return result.ready && result.status === expectedStatus;
    }
    
    /**
     * Click element and wait for HTMX to settle
     */
    static async clickAndWait(page: Page, selector: string, options: any = {}): Promise<boolean> {
        // Uses existing sophisticated implementation
        return await HTMXHelpers.clickAndWait(page, selector, options);
    }
    
    /**
     * Wait for log lines to appear using existing test API endpoints  
     */
    static async waitForLogs(
        page: Page, 
        minLines: number = 1, 
        options: any = {}
    ): Promise<string[]> {
        // Uses existing /api/test/wait-for-logs endpoint
        const result = await HTMXHelpers.waitForLogs(page, minLines, options);
        return result.logs;
    }
    
    /**
     * Legacy compatibility method
     */
    static async waitForHTMXRequest(page: Page): Promise<void> {
        const projectPath = (page.context() as any).projectPath || '/tmp/test-project';
        await HTMXHelpers.waitForSettle(page, { projectPath });
    }

    /**
     * Legacy compatibility method
     */
    static async checkForNavigation(page: Page, action: () => Promise<void>): Promise<boolean> {
        const initialUrl = page.url();
        await action();
        const projectPath = (page.context() as any).projectPath || '/tmp/test-project';
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 1000,
            projectPath 
        });
        return page.url() !== initialUrl;
    }

    // === Test Scenarios (replaces "fixtures" concept) ===
    
    /**
     * Setup empty project environment (for new user experience tests)
     */
    static async setupEmptyProject(): Promise<ProjectSetup> {
        await TestHelpers.initializeTestEnvironment(true);
        const baseUrl = await TestHelpers.startServer();
        
        return { 
            projects: [], 
            baseUrl,
            projectPaths: []
        };
    }
    
    /**
     * Setup single project environment (for standard tests)
     */
    static async setupSingleProject(language: string = 'rust'): Promise<ProjectSetup> {
        const projectName = await TestHelpers.generateTestProjectName('single');
        const projectPath = await TestHelpers.initProject(projectName, { language });
        await TestHelpers.addTool(projectPath, 'hello-world', language);
        
        const baseUrl = await TestHelpers.startServer();
        
        return { 
            projects: [{ name: projectName, language }], 
            baseUrl,
            projectPaths: [projectPath]
        };
    }
    
    /**
     * Setup multiple projects environment (for complex tests)
     */
    static async setupMultipleProjects(): Promise<ProjectSetup> {
        const projects = [];
        const projectPaths = [];
        
        for (const lang of ['rust', 'python', 'go']) {
            const name = await TestHelpers.generateTestProjectName(`multi-${lang}`);
            const projectPath = await TestHelpers.initProject(name, { language: lang });
            await TestHelpers.addTool(projectPath, `${name}-tool`, lang);
            
            projects.push({ name, language: lang });
            projectPaths.push(projectPath);
        }
        
        const baseUrl = await TestHelpers.startServer();
        
        return { projects, baseUrl, projectPaths };
    }
    
    // === Cleanup ===
    
    /**
     * Comprehensive cleanup using MCP tools
     */
    static async cleanup(): Promise<void> {
        try {
            const result = await TestHelpers.mcpClient.callTool('mcp-server__cleanup_test_projects', {
                remove_directories: true,
                base_path: TestHelpers.TEST_PROJECTS_DIR,
                max_age_minutes: 0 // Clean all projects
            });
            
            if (result.success) {
                console.log('Test projects cleaned up successfully');
            } else {
                console.warn('MCP cleanup failed:', result.error);
            }
        } catch (error) {
            console.warn('MCP cleanup error:', error);
        }
        
        // Fallback manual cleanup
        if (fs.existsSync(TestHelpers.TEST_PROJECTS_DIR)) {
            try {
                fs.rmSync(TestHelpers.TEST_PROJECTS_DIR, { 
                    recursive: true, 
                    force: true 
                });
                console.log('Manual cleanup completed');
            } catch (error) {
                console.warn('Manual cleanup failed:', error);
            }
        }
    }
    
    /**
     * Legacy compatibility method
     */
    static async cleanupTestData(): Promise<void> {
        await TestHelpers.cleanup();
    }
}

export default TestHelpers;