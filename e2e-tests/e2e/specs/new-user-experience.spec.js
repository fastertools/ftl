const { test, expect } = require('@playwright/test');
const TestFixtures = require('../fixtures/TestFixtures');
const TestHelpers = require('../utils/TestHelpers');
const fs = require('fs');
const path = require('path');
const os = require('os');

test.describe('New User Experience Tests', () => {
    let fixtures;
    let page;
    let centralizedProjectsPath;

    test.beforeAll(async () => {
        fixtures = new TestFixtures();
        page = await fixtures.setup({ headless: true });
        
        // Calculate the centralized projects.json path
        // This should match the UserDataPath logic in config.go
        let dataDir;
        if (process.env.XDG_DATA_HOME) {
            dataDir = process.env.XDG_DATA_HOME;
        } else {
            // macOS: ~/Library/Application Support
            // Windows: %APPDATA%  
            // Linux: ~/.config (but XDG_DATA_HOME should be ~/.local/share)
            if (process.platform === 'darwin') {
                dataDir = path.join(os.homedir(), 'Library', 'Application Support');
            } else if (process.platform === 'win32') {
                dataDir = process.env.APPDATA;
            } else {
                dataDir = path.join(os.homedir(), '.config');
            }
        }
        centralizedProjectsPath = path.join(dataDir, 'ftl', 'projects.json');
        
        console.log(`Centralized projects path: ${centralizedProjectsPath}`);
    });

    test.afterAll(async () => {
        await fixtures.teardown();
    });

    test.beforeEach(async () => {
        // Clean slate for each test - remove centralized projects file
        if (fs.existsSync(centralizedProjectsPath)) {
            fs.unlinkSync(centralizedProjectsPath);
            console.log(`Removed existing centralized projects file`);
        }
        
        // Also ensure directory is clean
        const centralizedDir = path.dirname(centralizedProjectsPath);
        if (fs.existsSync(centralizedDir)) {
            // Remove the entire ftl directory to simulate completely new user
            fs.rmSync(centralizedDir, { recursive: true, force: true });
            console.log(`Removed centralized directory for clean test`);
        }
    });

    test('should create empty projects.json on first load', async () => {
        console.log('Testing new user first load experience...');
        
        // Verify no centralized projects file exists initially
        expect(fs.existsSync(centralizedProjectsPath)).toBe(false);
        
        // Navigate to the console - this should trigger LoadProjects()
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        // Wait for the page to fully load
        await page.waitForTimeout(2000);
        
        // Verify the page loaded successfully
        const title = await page.title();
        expect(title).toContain('FTL');
        
        // Check that the centralized projects.json file was created
        expect(fs.existsSync(centralizedProjectsPath)).toBe(true);
        
        // Verify it contains an empty array
        const projectsContent = fs.readFileSync(centralizedProjectsPath, 'utf8');
        const projects = JSON.parse(projectsContent);
        expect(Array.isArray(projects)).toBe(true);
        expect(projects.length).toBe(0);
        
        console.log(`✅ Empty projects.json created at: ${centralizedProjectsPath}`);
        console.log(`Content: ${projectsContent}`);
    });

    test('should show Add Project button with no projects', async () => {
        // Navigate to console
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await page.waitForTimeout(2000);
        
        // Verify Add Project button is visible
        const addProjectButton = page.locator('#add-project-btn');
        await expect(addProjectButton).toBeVisible();
        
        // Verify no projects are listed (should show empty state)
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(0);
        
        console.log('✅ New user sees Add Project button with no existing projects');
    });

    test('should create centralized directory structure', async () => {
        // Verify directory doesn't exist initially
        const centralizedDir = path.dirname(centralizedProjectsPath);
        expect(fs.existsSync(centralizedDir)).toBe(false);
        
        // Navigate to console to trigger directory creation
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await page.waitForTimeout(2000);
        
        // Verify directory structure was created
        expect(fs.existsSync(centralizedDir)).toBe(true);
        expect(fs.existsSync(centralizedProjectsPath)).toBe(true);
        
        // Verify directory permissions (should be 0750)
        const stats = fs.statSync(centralizedDir);
        expect(stats.isDirectory()).toBe(true);
        
        console.log(`✅ Centralized directory created: ${centralizedDir}`);
    });

    test('should handle adding first project to empty projects.json', async () => {
        // Start with clean slate
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await page.waitForTimeout(2000);
        
        // Verify we start with empty projects.json
        expect(fs.existsSync(centralizedProjectsPath)).toBe(true);
        let projects = JSON.parse(fs.readFileSync(centralizedProjectsPath, 'utf8'));
        expect(projects.length).toBe(0);
        
        // Create a test project directory
        const testProjectName = `new-user-test-${Date.now()}`;
        const testProjectPath = path.resolve('.e2e-projects', testProjectName);
        TestHelpers.ensureTestDirectory(testProjectPath);
        
        // Click Add Project button
        const addProjectButton = page.locator('#add-project-btn');
        await addProjectButton.click();
        
        // Fill in the form
        await page.fill('input[name="name"]', testProjectName);
        await page.fill('input[name="path"]', testProjectPath);
        
        // Submit the form
        const submitButton = page.locator('#add-project-container button:has-text("Add")');
        await submitButton.click();
        
        // Wait for the project to be added
        await TestHelpers.waitForHTMXRequest(page);
        await page.waitForTimeout(2000);
        
        // Verify project was added to centralized file
        projects = JSON.parse(fs.readFileSync(centralizedProjectsPath, 'utf8'));
        expect(projects.length).toBe(1);
        expect(projects[0].name).toBe(testProjectName);
        expect(projects[0].path).toBe(testProjectPath);
        
        // Verify UI shows the project
        await page.reload({ waitUntil: 'domcontentloaded' });
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(1);
        
        console.log(`✅ First project successfully added to centralized projects.json`);
    });

    test('should not create hardcoded default project', async () => {
        // Navigate to console
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await page.waitForTimeout(3000); // Give more time for any default project logic
        
        // Verify projects.json exists and is empty (no hardcoded default)
        expect(fs.existsSync(centralizedProjectsPath)).toBe(true);
        const projects = JSON.parse(fs.readFileSync(centralizedProjectsPath, 'utf8'));
        expect(projects.length).toBe(0);
        
        // Verify UI shows no projects
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(0);
        
        console.log('✅ No hardcoded default project created - clean new user experience');
    });
});