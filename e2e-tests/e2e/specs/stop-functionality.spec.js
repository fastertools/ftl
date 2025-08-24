const { test, expect } = require('@playwright/test');
const DashboardPage = require('../pages/DashboardPage');
const TestFixtures = require('../fixtures/TestFixtures');
const HTMXHelpers = require('../utils/HTMXHelpers');

test.describe('Stop Functionality Tests', () => {
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

    test('should handle stop buttons without throwing unknown tool errors', async () => {
        // This test verifies that stop buttons use the correct unified MCP tool name
        // and don't throw "unknown tool mcp-server__stop_regular" or "mcp-server__stop_watch" errors
        
        // Test the process control interface for proper tool names
        await page.waitForSelector('#ftl-status', { timeout: 5000 });
        
        // Look for stop buttons (they may not be present if no process is running)
        const stopButtons = await page.locator('button').filter({ hasText: 'Stop' });
        const stopButtonCount = await stopButtons.count();
        
        console.log(`Found ${stopButtonCount} stop buttons on the page`);
        
        if (stopButtonCount > 0) {
            // If stop buttons exist, click one and verify no "unknown tool" error appears
            const firstStopButton = stopButtons.first();
            
            // Listen for the response to check for errors
            let errorOccurred = false;
            page.on('response', response => {
                if (response.url().includes('/htmx/process/stop')) {
                    console.log(`Stop request status: ${response.status()}`);
                }
            });
            
            // Monitor for error messages in the operation feedback area
            const operationFeedback = page.locator('#operation-feedback');
            
            // Click the stop button
            await firstStopButton.click();
            
            // Wait for the stop operation to complete
            await HTMXHelpers.waitForSettle(page, { 
                timeout: 3000,
                projectPath: page.context().projectPath 
            });
            
            // Check if any error message appeared
            const feedbackText = await operationFeedback.textContent();
            console.log(`Operation feedback after stop: "${feedbackText}"`);
            
            // Verify no "unknown tool" error occurred
            expect(feedbackText).not.toContain('unknown tool');
            expect(feedbackText).not.toContain('mcp-server__stop_regular');
            expect(feedbackText).not.toContain('mcp-server__stop_watch');
            
            // If there's an error, it should be a legitimate error (like "no process running")
            // not a tool resolution error
            if (feedbackText.includes('Error:') || feedbackText.includes('failed:')) {
                expect(feedbackText).not.toContain('JSON RPC invalid params');
                expect(feedbackText).not.toContain('unknown tool');
            }
        } else {
            console.log('No stop buttons found - no active processes to test');
            // This is fine - if no processes are running, no stop buttons should be present
        }
    });

    test('should use correct MCP tool names in HTMX requests', async () => {
        // This test verifies that HTMX requests use the unified tool names
        
        // Check the HTML for proper hx-vals attributes
        const stopButtons = await page.locator('button[hx-post="/htmx/process/stop"]');
        const stopButtonCount = await stopButtons.count();
        
        if (stopButtonCount > 0) {
            // Get the hx-vals attribute from the first stop button
            const hxVals = await stopButtons.first().getAttribute('hx-vals');
            console.log(`Stop button hx-vals: ${hxVals}`);
            
            // The hx-vals should contain process_type but the backend should
            // map this to the unified mcp-server__stop tool
            expect(hxVals).toBeTruthy();
            expect(hxVals).toContain('process_type');
        }
    });

    test('should display appropriate messages for stop operations', async () => {
        // Test that stop operations provide appropriate feedback
        const stopButtons = await page.locator('button').filter({ hasText: 'Stop' });
        const stopButtonCount = await stopButtons.count();
        
        if (stopButtonCount > 0) {
            const operationFeedback = page.locator('#operation-feedback');
            
            // Clear any existing feedback
            await page.evaluate(() => {
                const feedback = document.getElementById('operation-feedback');
                if (feedback) feedback.innerHTML = '';
            });
            
            // Click stop button
            await stopButtons.first().click();
            
            // Wait for response
            await HTMXHelpers.waitForSettle(page, { 
                timeout: 2000,
                projectPath: page.context().projectPath 
            });
            
            const feedbackText = await operationFeedback.textContent();
            console.log(`Stop operation result: "${feedbackText}"`);
            
            // Should either show success (empty) or a legitimate error message
            // But never an "unknown tool" error
            expect(feedbackText).not.toContain('unknown tool');
            expect(feedbackText).not.toContain('mcp-server__stop_regular');
            expect(feedbackText).not.toContain('mcp-server__stop_watch');
        }
    });
});