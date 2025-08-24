const { test, expect } = require('@playwright/test');
const TestFixtures = require('../fixtures/TestFixtures');
const TestHelpers = require('../utils/TestHelpers');
const HTMXHelpers = require('../utils/HTMXHelpers');
const fs = require('fs');
const path = require('path');
const os = require('os');

test.describe('New User Experience Tests', () => {
    let fixtures;
    let page;
    let testProjectsPath;

    test.beforeAll(async () => {
        // Use standardized test projects file location
        testProjectsPath = path.join(process.cwd(), '.e2e-projects.json');
        
        // Clean slate - remove file to simulate new user
        if (fs.existsSync(testProjectsPath)) {
            fs.unlinkSync(testProjectsPath);
            console.log(`Removed existing test projects file for new user testing`);
        }
        
        // Setup fixtures WITHOUT creating test project
        fixtures = new TestFixtures();
        page = await fixtures.setupWithoutProject({ headless: true });
        
        console.log(`Test projects path: ${testProjectsPath}`);
    });

    test.afterAll(async () => {
        await fixtures.teardown();
    });

    test.beforeEach(async () => {
        // Ensure clean state for each test
        if (fs.existsSync(testProjectsPath)) {
            fs.unlinkSync(testProjectsPath);
        }
    });

    test('should create empty projects.json on first load', async () => {
        console.log('Testing new user first load experience...');
        
        // Verify file doesn't exist initially (simulating new user)
        expect(fs.existsSync(testProjectsPath)).toBe(false);
        
        // Navigate to the console - this should trigger LoadProjects()
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        // Wait for the page to fully load and server to create the file
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 2000,
            projectPath: testProjectsPath 
        });
        
        // Verify the page loaded successfully
        const title = await page.title();
        expect(title).toContain('FTL');
        
        // Check that the projects.json file was created by the server
        expect(fs.existsSync(testProjectsPath)).toBe(true);
        
        // Verify it contains an empty array
        const projectsContent = fs.readFileSync(testProjectsPath, 'utf8');
        const projects = JSON.parse(projectsContent);
        expect(Array.isArray(projects)).toBe(true);
        expect(projects.length).toBe(0);
        
        console.log(`✅ Empty projects.json created at: ${testProjectsPath}`);
        console.log(`Content: ${projectsContent}`);
    });

    test('should show Add Project button with no projects', async () => {
        // Navigate to console
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 2000,
            projectPath: testProjectsPath 
        });
        
        // Verify Add Project button is visible
        const addProjectButton = page.locator('#add-project-btn');
        await expect(addProjectButton).toBeVisible();
        
        // Verify no projects are listed (should show empty state)
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(0);
        
        console.log('✅ New user sees Add Project button with no existing projects');
    });

    test('should create test projects file in current directory', async () => {
        // Verify file doesn't exist initially
        expect(fs.existsSync(testProjectsPath)).toBe(false);
        
        // Navigate to console to trigger file creation
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 2000,
            projectPath: testProjectsPath 
        });
        
        // Verify the test projects file was created
        expect(fs.existsSync(testProjectsPath)).toBe(true);
        
        // Verify it's a file, not a directory
        const stats = fs.statSync(testProjectsPath);
        expect(stats.isFile()).toBe(true);
        
        console.log(`✅ Test projects file created: ${testProjectsPath}`);
    });

    test('should handle adding first project to empty projects.json', async () => {
        // Start with clean slate
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 2000,
            projectPath: testProjectsPath 
        });
        
        // Verify we start with empty projects.json
        expect(fs.existsSync(testProjectsPath)).toBe(true);
        let projects = JSON.parse(fs.readFileSync(testProjectsPath, 'utf8'));
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
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 2000,
            projectPath: testProjectsPath 
        });
        
        // Verify project was added to test projects file
        projects = JSON.parse(fs.readFileSync(testProjectsPath, 'utf8'));
        expect(projects.length).toBe(1);
        expect(projects[0].name).toBe(testProjectName);
        expect(projects[0].path).toBe(testProjectPath);
        
        // Verify UI shows the project
        await page.reload({ waitUntil: 'domcontentloaded' });
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(1);
        
        console.log(`✅ First project successfully added to test projects.json`);
    });

    test('should not create hardcoded default project', async () => {
        // Force server to reload projects from disk (which should be empty due to beforeEach)
        await fixtures.reloadServerProjects();
        
        // Navigate to console
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await HTMXHelpers.waitForSettle(page, { 
            timeout: 3000,  // Give more time for any default project logic
            projectPath: testProjectsPath 
        });
        
        // Verify projects.json exists and is empty (no hardcoded default)
        expect(fs.existsSync(testProjectsPath)).toBe(true);
        const projects = JSON.parse(fs.readFileSync(testProjectsPath, 'utf8'));
        expect(projects.length).toBe(0);
        
        // Verify UI shows no projects
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(0);
        
        console.log('✅ No hardcoded default project created - clean new user experience');
    });
});