const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

class TestHelpers {
    static async resetTestProjectsFile() {
        // Create .e2e-projects workspace directory
        const e2eWorkspace = path.resolve('.e2e-projects');
        if (!fs.existsSync(e2eWorkspace)) {
            fs.mkdirSync(e2eWorkspace, { recursive: true });
        }
        
        // Create unique test project for this test run
        const projectName = `test-project-${Date.now()}`;
        const projectPath = path.join(e2eWorkspace, projectName);
        
        // Initialize as valid FTL project using built CLI
        TestHelpers.ensureTestDirectory(projectPath);
        
        const testProject = {
            name: projectName,
            path: projectPath,
            added_at: new Date().toISOString(),
            last_active: new Date().toISOString()
        };
        
        const fsPromises = require('fs').promises;
        await fsPromises.writeFile('test_projects.json', JSON.stringify([testProject], null, 2));
        console.log(`resetTestProjectsFile: Created test project at ${projectPath}`);
    }

    static async startServer(useTestProjects = true) {
        const env = { ...process.env };
        if (useTestProjects) {
            env.PROJECTS_FILE = 'test_projects.json';
        }
        
        // Use the built binary from current branch
        const serverProcess = spawn('./bin/ftl', ['dev', 'console', '--port', '8080'], {
            env,
            stdio: ['ignore', 'pipe', 'pipe']
        });

        // Wait for server to start
        await new Promise((resolve) => {
            setTimeout(resolve, 3000);
        });

        return serverProcess;
    }

    static async stopServer(serverProcess) {
        if (serverProcess) {
            serverProcess.kill('SIGTERM');
        }
    }

    static generateTestProjectName(prefix = 'TestProject') {
        return `${prefix}_${Date.now()}`;
    }

    static generateTestProjectPath(name) {
        // Use a test data directory with unique subdirectories
        // We'll create this directory if it doesn't exist
        const testDir = `./test_data/${name.toLowerCase().replace(/\s+/g, '_')}`;
        return testDir;
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
        await page.waitForTimeout(1000); // Give HTMX time to process
    }

    static async checkForNavigation(page, action) {
        const initialUrl = page.url();
        await action();
        await page.waitForTimeout(500);
        return page.url() !== initialUrl;
    }

    static async cleanupTestData() {
        // Remove test_projects.json
        if (fs.existsSync('test_projects.json')) {
            fs.unlinkSync('test_projects.json');
            console.log('cleanupTestData: Removed test_projects.json');
        }
        
        // Remove .e2e-projects directory
        const e2eWorkspace = path.resolve('.e2e-projects');
        if (fs.existsSync(e2eWorkspace)) {
            fs.rmSync(e2eWorkspace, { recursive: true, force: true });
            console.log('cleanupTestData: Removed .e2e-projects directory');
        }
    }
}

module.exports = TestHelpers;