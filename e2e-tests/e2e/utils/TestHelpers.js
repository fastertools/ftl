const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const HTMXHelpers = require('./HTMXHelpers');
const TestLifecycle = require('./TestLifecycle');
const TestDataManager = require('../data/TestDataManager');
const TestDataFactory = require('../data/TestDataFactory');

class TestHelpers {
    static async resetTestProjectsFile(createEmpty = false) {
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

    static async startServer(useTestProjects = true) {
        const env = { ...process.env };
        if (useTestProjects) {
            env.PROJECTS_FILE = 'test_projects.json';
        }
        
        // Enable test mode for the server
        env.FTL_TEST_MODE = 'true';
        
        // Use the built binary from current branch
        const serverProcess = spawn('./bin/ftl', ['dev', 'console', '--port', '8080'], {
            env,
            stdio: ['ignore', 'pipe', 'pipe']
        });

        // Use TestLifecycle for deterministic server ready wait
        // Get the project path from environment or use default
        const projectPath = env.FTL_PROJECT_PATH || '/tmp/test-project';
        const isReady = await TestLifecycle.waitForReady(projectPath, {
            timeout: 30000,
            maxAttempts: 10,
            mcpServerPath: './mcp-server'
        });
        
        if (!isReady) {
            console.warn('Server may not be fully ready after timeout');
        }

        return serverProcess;
    }

    static async stopServer(serverProcess) {
        if (serverProcess) {
            serverProcess.kill('SIGTERM');
        }
    }

    static generateTestProjectName(prefix = 'TestProject') {
        // Use standardized ID generation
        return TestDataFactory.generateId(prefix);
    }

    static generateTestProjectPath(name) {
        // Use standardized path generation
        const fixture = TestDataManager.getCurrentFixture();
        return path.join(fixture.basePath, name.toLowerCase().replace(/\s+/g, '_'));
    }

    static ensureTestDirectory(testPath) {
        // Create the test directory if it doesn't exist
        if (!fs.existsSync(testPath)) {
            fs.mkdirSync(testPath, { recursive: true });
            
            // Initialize as a valid FTL project
            const { execSync } = require('child_process');
            try {
                // Extract project name from path for ftl init using built CLI
                const projectName = path.basename(testPath);
                execSync(`../bin/ftl init ${projectName}`, { 
                    cwd: testPath,
                    stdio: 'ignore' // Suppress output
                });
            } catch (error) {
                // If ftl init fails, create a minimal ftl.toml manually
                const ftlTomlContent = `name = "${path.basename(testPath)}"

[runtime]
language = "go"
`;
                fs.writeFileSync(path.join(testPath, 'ftl.toml'), ftlTomlContent);
            }
        }
    }

    static async takeScreenshot(page, name) {
        await page.screenshot({ 
            path: `e2e-tests/e2e/screenshots/${name}.png`,
            fullPage: true 
        });
    }

    static async waitForHTMXRequest(page) {
        // Use HTMXHelpers for deterministic wait instead of arbitrary timeout
        const projectPath = page.context().projectPath || '/tmp/test-project';
        await HTMXHelpers.waitForSettle(page, { projectPath });
    }

    static async checkForNavigation(page, action) {
        const initialUrl = page.url();
        await action();
        // Use HTMXHelpers instead of arbitrary timeout
        const projectPath = page.context().projectPath || '/tmp/test-project';
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 1000,
            projectPath 
        });
        return page.url() !== initialUrl;
    }

    static async cleanupTestData() {
        // Use standardized cleanup
        await TestDataManager.cleanup();
    }
}

module.exports = TestHelpers;