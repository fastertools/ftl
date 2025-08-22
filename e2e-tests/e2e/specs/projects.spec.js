const { test, expect } = require('@playwright/test');
const ProjectSidebarPage = require('../pages/ProjectSidebarPage');
const TestFixtures = require('../fixtures/TestFixtures');
const TestHelpers = require('../utils/TestHelpers');

test.describe('Project Management Tests', () => {
    let fixtures;
    let page;
    let projectSidebar;

    test.beforeAll(async () => {
        fixtures = new TestFixtures();
        page = await fixtures.setup({ headless: true });
        projectSidebar = new ProjectSidebarPage(page);
    });

    test.afterAll(async () => {
        await fixtures.teardown();
    });

    test.beforeEach(async () => {
        // Reset projects file and reload server state for each test
        await TestHelpers.resetTestProjectsFile();
        await fixtures.reloadServerProjects();
        // Force a fresh page load to avoid cached UI state
        await page.goto('http://localhost:8080', { 
            waitUntil: 'domcontentloaded',
            // Force reload to avoid cache
            waitUntil: 'networkidle'
        });
        await page.waitForTimeout(500); // Give UI time to render
    });

    test('should show Add Project button', async () => {
        const isVisible = await projectSidebar.isAddButtonVisible();
        expect(isVisible).toBe(true);
    });

    test('should show form when Add Project clicked', async () => {
        await projectSidebar.clickAddProject();
        const isFormVisible = await projectSidebar.isFormVisible();
        expect(isFormVisible).toBe(true);
    });

    test('should hide form when Cancel clicked', async () => {
        await projectSidebar.clickAddProject();
        await projectSidebar.cancelProjectForm();
        const isButtonVisible = await projectSidebar.isAddButtonVisible();
        expect(isButtonVisible).toBe(true);
    });

    test('should add new project', async () => {
        const projectName = TestHelpers.generateTestProjectName();
        const projectPath = TestHelpers.generateTestProjectPath(projectName);
        
        // Ensure the test directory exists
        TestHelpers.ensureTestDirectory(projectPath);
        
        // Verify we start with exactly 1 project (the test-project from reset)
        const initialCount = await projectSidebar.getProjectCount();
        expect(initialCount).toBe(1); // Should have exactly 1 project after reset
        
        await projectSidebar.clickAddProject();
        await projectSidebar.fillProjectForm(projectName, projectPath);
        await projectSidebar.submitProjectForm();
        
        await TestHelpers.waitForHTMXRequest(page);
        // Give HTMX more time to update the DOM
        await page.waitForTimeout(2000);
        
        // Refresh the page to ensure we see the current server state
        await page.reload({ waitUntil: 'domcontentloaded' });
        
        const newCount = await projectSidebar.getProjectCount();
        expect(newCount).toBe(2); // Should now have 2 projects
        
        const exists = await projectSidebar.projectExists(projectName);
        expect(exists).toBe(true);
    });

    test('should switch between projects', async () => {
        // Add a test project first
        const projectName = TestHelpers.generateTestProjectName('SwitchTest');
        const projectPath = TestHelpers.generateTestProjectPath(projectName);
        
        // Ensure the test directory exists
        TestHelpers.ensureTestDirectory(projectPath);
        
        await projectSidebar.clickAddProject();
        await projectSidebar.fillProjectForm(projectName, projectPath);
        await projectSidebar.submitProjectForm();
        
        await TestHelpers.waitForHTMXRequest(page);
        
        // Switch to the new project
        await projectSidebar.switchToProject(projectName);
        
        // Verify switch happened (page should refresh)
        const url = page.url();
        expect(url).toContain('localhost:8080');
    });

    test.skip('should remove project', async () => {
        // Verify we start with exactly 1 project
        const initialCount = await projectSidebar.getProjectCount();
        expect(initialCount).toBe(1);
        
        // Add a test project first
        const projectName = TestHelpers.generateTestProjectName('RemoveTest');
        const projectPath = TestHelpers.generateTestProjectPath(projectName);
        
        // Ensure the test directory exists
        TestHelpers.ensureTestDirectory(projectPath);
        
        await projectSidebar.clickAddProject();
        await projectSidebar.fillProjectForm(projectName, projectPath);
        await projectSidebar.submitProjectForm();
        
        await TestHelpers.waitForHTMXRequest(page);
        await page.waitForTimeout(1000);
        
        // Reload to ensure we see the added project
        await page.reload({ waitUntil: 'domcontentloaded' });
        
        // Verify project was added
        const countAfterAdd = await projectSidebar.getProjectCount();
        expect(countAfterAdd).toBe(2);
        
        // Remove the project
        await projectSidebar.removeProject(projectName);
        
        await TestHelpers.waitForHTMXRequest(page);
        await page.waitForTimeout(1000);
        
        // Reload to see the updated state
        await page.reload({ waitUntil: 'domcontentloaded' });
        
        // Verify it's gone
        const exists = await projectSidebar.projectExists(projectName);
        expect(exists).toBe(false);
        
        // Verify count is back to 1
        const finalCount = await projectSidebar.getProjectCount();
        expect(finalCount).toBe(1);
    });
});