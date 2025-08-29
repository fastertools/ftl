import * as fs from 'fs';
import * as path from 'path';
import { spawn, execSync, ChildProcess } from 'child_process';
import { Page } from '@playwright/test';
import HTMXHelpers from './htmx-helpers';
import ServerManager from './server-manager';

interface ServerOptions {
    useTestProjects?: boolean;
    timeout?: number;
    retries?: number;
}

// These appear to be external dependencies that might not be migrated yet
declare const TestDataManager: any;
declare const TestDataFactory: any;

class TestHelpers {
    // Project paths
    static readonly PROJECT_ROOT = path.resolve(__dirname, '..', '..', '..');
    static readonly TEST_PROJECTS_DIR = path.resolve(__dirname, '..', '.e2e-projects');
    static readonly TEST_PROJECTS_FILE = path.join(TestHelpers.TEST_PROJECTS_DIR, '.e2e-projects.json');
    static readonly FTL_BINARY = path.join(TestHelpers.PROJECT_ROOT, 'bin', 'ftl');
    static readonly SCREENSHOTS_DIR = path.join(TestHelpers.PROJECT_ROOT, 'ui', 'e2e', 'screenshots');
    static readonly TEST_CONFIG_FILE = path.join(__dirname, '.test-config.json');

    /**
     * Verify that the FTL binary exists
     */
    static verifyFtlBinaryExists(): void {
        if (!fs.existsSync(TestHelpers.FTL_BINARY)) {
            throw new Error(
                `FTL binary not found at ${TestHelpers.FTL_BINARY}. ` +
                `Please build FTL first by running 'make build' from ${TestHelpers.PROJECT_ROOT}`
            );
        }
    }
    static async resetTestProjectsFile(createEmpty: boolean = false): Promise<void> {
        // Use standardized test data system
        if (createEmpty) {
            // For new-user tests - use empty fixture
            await TestDataManager.initialize('empty');
        } else {
            // For normal tests - use single project fixture
            await TestDataManager.initialize('single');
        }
        
        const fixture = TestDataManager.getCurrentFixture();
        console.log(`Test data initialized: ${fixture.metadata.fixture} fixture`);
    }

    static async startServer(options: ServerOptions = {}): Promise<string> {
        const serverManager = new ServerManager();
        return await serverManager.startServer(options);
    }

    static async stopServer(serverProcess?: ChildProcess): Promise<void> {
        if (serverProcess) {
            serverProcess.kill('SIGTERM');
        }
    }

    static generateTestProjectName(prefix: string = 'TestProject'): string {
        // Use standardized ID generation
        return TestDataFactory.generateId(prefix);
    }

    static generateTestProjectPath(name: string): string {
        // Use standardized path generation
        const fixture = TestDataManager.getCurrentFixture();
        return path.join(fixture.basePath, name.toLowerCase().replace(/\s+/g, '_'));
    }

    static ensureTestDirectory(testPath: string): void {
        // Create the test directory if it doesn't exist
        if (!fs.existsSync(testPath)) {
            fs.mkdirSync(testPath, { recursive: true });
            
            // Initialize as a valid FTL project
            try {
                // Extract project name from path for ftl init using built CLI
                const projectName = path.basename(testPath);
                execSync(`${TestHelpers.FTL_BINARY} init ${projectName}`, { 
                    cwd: testPath,
                    stdio: 'ignore' // Suppress output
                });

                execSync(`cd ${projectName} && ${TestHelpers.FTL_BINARY} add ${projectName}-tool -l rust`, { 
                    cwd: testPath,
                    stdio: 'ignore' // Suppress output
                });
                
            } catch (error) {
                throw new Error(`ftl init failed with: ${error}`)
            }
        }
    }

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

    static async waitForHTMXRequest(page: Page): Promise<void> {
        // Use HTMXHelpers for deterministic wait instead of arbitrary timeout
        const projectPath = (page.context() as any).projectPath || '/tmp/test-project';
        await HTMXHelpers.waitForSettle(page, { projectPath });
    }

    static async checkForNavigation(page: Page, action: () => Promise<void>): Promise<boolean> {
        const initialUrl = page.url();
        await action();
        // Use HTMXHelpers instead of arbitrary timeout
        const projectPath = (page.context() as any).projectPath || '/tmp/test-project';
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 1000,
            projectPath 
        });
        return page.url() !== initialUrl;
    }

    static async cleanupTestData(): Promise<void> {
        // Use standardized cleanup
        await TestDataManager.cleanup();
    }
}

export default TestHelpers;