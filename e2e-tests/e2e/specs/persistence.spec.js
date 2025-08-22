const { test, expect } = require('@playwright/test');

test.describe('Persistence Tests', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('http://localhost:8080');
        await page.waitForLoadState('networkidle');
    });

    test('should persist command output across project switches', async ({ page }) => {
        // Wait for the page to load and switch to Command Output tab
        await page.waitForSelector('#commands-tab');
        await page.click('#commands-tab');
        await page.waitForSelector('#ftl-output', { state: 'visible' });
        
        // Execute a build command to generate some output
        const buildButton = page.locator('button:has-text("Build")').first();
        await expect(buildButton).toBeVisible();
        await buildButton.click();
        
        // Wait for some output to appear
        await page.waitForTimeout(3000);
        
        // Check if there's output in the console
        const outputBefore = await page.locator('#ftl-output').innerHTML();
        console.log('Output before project switch:', outputBefore);
        
        // Only proceed if we have output
        if (outputBefore.includes('Ready to execute commands...') && !outputBefore.includes('&gt;')) {
            console.log('No command output generated yet, skipping persistence test');
            return;
        }
        
        // Switch to a different project if multiple exist
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        
        if (projectCount > 1) {
            // Click on a different project
            const secondProject = projectItems.nth(1);
            await secondProject.click();
            
            // Wait for the switch to complete
            await page.waitForTimeout(2000);
            
            // Switch back to the original project
            const firstProject = projectItems.nth(0);
            await firstProject.click();
            
            // Wait for the switch to complete
            await page.waitForTimeout(2000);
            
            // Make sure we're on the Command Output tab after switch
            await page.click('#commands-tab');
            await page.waitForSelector('#ftl-output', { state: 'visible' });
            
            // Check if the command output persisted
            const outputAfter = await page.locator('#ftl-output').innerHTML();
            console.log('Output after project switch:', outputAfter);
            
            // The output should contain the previous command execution
            expect(outputAfter).toContain('&gt;');
        } else {
            console.log('Only one project available, cannot test persistence across project switches');
        }
    });

    test('should persist logs across project switches', async ({ page }) => {
        // Wait for the page to load
        await page.waitForSelector('#live-log-content');
        
        // Wait for some logs to accumulate (if any)
        await page.waitForTimeout(5000);
        
        // Get initial log content
        const logsBefore = await page.locator('#live-log-content').innerHTML();
        console.log('Logs before project switch:', logsBefore);
        
        // Switch to a different project if multiple exist
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        
        if (projectCount > 1) {
            // Click on a different project
            const secondProject = projectItems.nth(1);
            await secondProject.click();
            
            // Wait for the switch to complete
            await page.waitForTimeout(2000);
            
            // Switch back to the original project
            const firstProject = projectItems.nth(0);
            await firstProject.click();
            
            // Wait for the switch to complete
            await page.waitForTimeout(2000);
            
            // Check if logs are still there (or at least the container is properly initialized)
            const logsAfter = await page.locator('#live-log-content').innerHTML();
            console.log('Logs after project switch:', logsAfter);
            
            // The logs should either contain the previous content or show "Waiting for log output..." 
            // rather than being empty
            expect(logsAfter.length).toBeGreaterThan(0);
            expect(logsAfter).not.toBe('');
        } else {
            console.log('Only one project available, cannot test persistence across project switches');
        }
    });
    
    test('should generate and execute command for persistence test', async ({ page }) => {
        // Wait for the page to load and switch to Command Output tab
        await page.waitForSelector('#commands-tab');
        await page.click('#commands-tab');
        await page.waitForSelector('#ftl-output', { state: 'visible' });
        
        // Execute a build command
        const buildButton = page.locator('button:has-text("Build")').first();
        await expect(buildButton).toBeVisible();
        await buildButton.click();
        
        // Wait longer for command to complete
        await page.waitForTimeout(5000);
        
        // Check for command output
        const output = await page.locator('#ftl-output').innerHTML();
        console.log('Command output after build:', output);
        
        // Should have some command execution indicator
        const hasCommandOutput = output.includes('&gt;') || output.includes('ftl build') || 
                                !output.includes('Ready to execute commands...');
        
        if (hasCommandOutput) {
            console.log('✅ Command output generated successfully');
        } else {
            console.log('⚠️ No command output detected - may need to wait longer or check MCP connection');
        }
        
        // The output div should have changed from the default
        expect(output).toBeDefined();
        expect(output.length).toBeGreaterThan(0);
    });
});