const { test, expect } = require('@playwright/test');
const DashboardPage = require('../pages/DashboardPage');
const TestFixtures = require('../fixtures/TestFixtures');

test.describe('Activity Panel Tab Tests', () => {
  let fixtures;
  let page;
  let dashboardPage;

  test.beforeAll(async () => {
    fixtures = new TestFixtures();
    page = await fixtures.setup({ headless: true });
    dashboardPage = new DashboardPage(page);
  });

  test.afterAll(async () => {
    await fixtures.teardown();
  });

  test.beforeEach(async () => {
    await dashboardPage.navigate();
  });

  test('should have Activity panel with tabs', async () => {
    // Check that Activity panel exists
    const activityPanel = await page.locator('h2:has-text("Activity")');
    await expect(activityPanel).toBeVisible();
    
    // Check that both tabs exist
    const logsTab = await page.locator('#logs-tab');
    const commandsTab = await page.locator('#commands-tab');
    
    await expect(logsTab).toBeVisible();
    await expect(commandsTab).toBeVisible();
    
    // Check tab text
    await expect(logsTab).toHaveText('Live Logs');
    await expect(commandsTab).toHaveText('Command Output');
  });

  test('should have Live Logs tab active by default', async () => {
    const logsTab = await page.locator('#logs-tab');
    const commandsTab = await page.locator('#commands-tab');
    const logsContent = await page.locator('#logs-content');
    const commandsContent = await page.locator('#commands-content');
    
    // Check that logs tab is active (has border-blue-500 class)
    const logsTabClasses = await logsTab.getAttribute('class');
    expect(logsTabClasses).toContain('border-blue-500');
    expect(logsTabClasses).toContain('text-white');
    
    // Check that commands tab is inactive
    const commandsTabClasses = await commandsTab.getAttribute('class');
    expect(commandsTabClasses).toContain('border-transparent');
    
    // Check that logs content is visible
    await expect(logsContent).toBeVisible();
    
    // Check that commands content is hidden
    const commandsDisplay = await commandsContent.evaluate(el => 
      window.getComputedStyle(el).display
    );
    expect(commandsDisplay).toBe('none');
  });

  test('should switch tabs when clicked', async () => {
    const logsTab = await page.locator('#logs-tab');
    const commandsTab = await page.locator('#commands-tab');
    const logsContent = await page.locator('#logs-content');
    const commandsContent = await page.locator('#commands-content');
    
    // Click on Command Output tab
    await commandsTab.click();
    
    // Wait a moment for the switch
    await page.waitForTimeout(100);
    
    // Check that commands tab is now active
    const commandsTabClasses = await commandsTab.getAttribute('class');
    expect(commandsTabClasses).toContain('border-blue-500');
    expect(commandsTabClasses).toContain('text-white');
    
    // Check that logs tab is now inactive
    const logsTabClasses = await logsTab.getAttribute('class');
    expect(logsTabClasses).toContain('border-transparent');
    
    // Check that commands content is visible
    const commandsDisplay = await commandsContent.evaluate(el => 
      window.getComputedStyle(el).display
    );
    expect(commandsDisplay).toBe('block');
    
    // Check that logs content is hidden
    const logsDisplay = await logsContent.evaluate(el => 
      window.getComputedStyle(el).display
    );
    expect(logsDisplay).toBe('none');
    
    // Switch back to logs
    await logsTab.click();
    await page.waitForTimeout(100);
    
    // Verify it switched back
    const logsDisplayAfter = await logsContent.evaluate(el => 
      window.getComputedStyle(el).display
    );
    expect(logsDisplayAfter).toBe('block');
  });

  test('should auto-switch to Command Output tab when command is executed', async () => {
    const commandsTab = await page.locator('#commands-tab');
    const commandsContent = await page.locator('#commands-content');
    
    // Verify we start on Live Logs tab
    const initialCommandsDisplay = await commandsContent.evaluate(el => 
      window.getComputedStyle(el).display
    );
    expect(initialCommandsDisplay).toBe('none');
    
    // Execute a build command
    const buildButton = await page.locator('button:has-text("Build")');
    await expect(buildButton).toBeVisible();
    await buildButton.click();
    
    // Wait for the command to execute and tab to switch
    await page.waitForTimeout(1000);
    
    // Check that we switched to Command Output tab
    const commandsDisplay = await commandsContent.evaluate(el => 
      window.getComputedStyle(el).display
    );
    expect(commandsDisplay).toBe('block');
    
    // Check that commands tab is active
    const commandsTabClasses = await commandsTab.getAttribute('class');
    expect(commandsTabClasses).toContain('border-blue-500');
    expect(commandsTabClasses).toContain('text-white');
    
    // Check that command output is visible
    const ftlOutput = await page.locator('#ftl-output');
    await expect(ftlOutput).toBeVisible();
  });

  test('should maintain log polling in Live Logs tab', async () => {
    // Switch to Live Logs tab
    const logsTab = await page.locator('#logs-tab');
    await logsTab.click();
    
    // Check that the log content div has polling attributes
    const logContent = await page.locator('#live-log-content');
    await expect(logContent).toBeVisible();
    
    const hxTrigger = await logContent.getAttribute('hx-trigger');
    expect(hxTrigger).toBe('every 2s');
    
    const hxPost = await logContent.getAttribute('hx-post');
    expect(hxPost).toBe('/htmx/logs/poll');
  });

  test('should have log control buttons in Live Logs tab', async () => {
    // Make sure we're on Live Logs tab
    const logsTab = await page.locator('#logs-tab');
    await logsTab.click();
    await page.waitForTimeout(100);
    
    // Check for Clear button
    const clearButton = await page.locator('button:has-text("Clear")');
    await expect(clearButton).toBeVisible();
    
    // Check for Scroll to Bottom button
    const scrollButton = await page.locator('button:has-text("Scroll to Bottom")');
    await expect(scrollButton).toBeVisible();
  });

  test('should show command history in Command Output tab', async () => {
    // Switch to Command Output tab
    const commandsTab = await page.locator('#commands-tab');
    await commandsTab.click();
    await page.waitForTimeout(100);
    
    // Check that ftl-output div exists
    const ftlOutput = await page.locator('#ftl-output');
    await expect(ftlOutput).toBeVisible();
    
    // Should either show "Ready to execute commands..." or actual commands
    const outputText = await ftlOutput.textContent();
    expect(outputText.length).toBeGreaterThan(0);
  });

  test('should have 2-column layout for top panels', async () => {
    // Check that the grid has 2 columns
    const gridContainer = await page.locator('.grid.grid-cols-2');
    await expect(gridContainer).toBeVisible();
    
    // Check that Project Info panel exists
    const projectInfo = await page.locator('h2:has-text("Project Info")');
    await expect(projectInfo).toBeVisible();
    
    // Check that Control Center panel exists
    const controlCenter = await page.locator('h2:has-text("Control Center")');
    await expect(controlCenter).toBeVisible();
  });

  test('should have Activity panel as full-width at bottom', async () => {
    // Activity panel should be in its own container below the grid
    const activityContainer = await page.locator('.px-6.pb-6 .panel');
    await expect(activityContainer).toBeVisible();
    
    // It should contain the Activity heading
    const activityHeading = await activityContainer.locator('h2:has-text("Activity")');
    await expect(activityHeading).toBeVisible();
    
    // It should not be inside the grid-cols-2
    const gridContainer = await page.locator('.grid.grid-cols-2');
    const activityInGrid = await gridContainer.locator('h2:has-text("Activity")').count();
    expect(activityInGrid).toBe(0);
  });
});