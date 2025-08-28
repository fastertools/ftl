/**
 * Control Operations Integration Workflow Tests
 * 
 * Tests the complete control operations workflow including Build, Up, Watch operations,
 * HTMX requests, DOM updates, message injection, and polling interactions.
 */

const { test, expect } = require('@playwright/test');
const { 
  renderBaseButton,
  setupTestProjectsInSidebar,
  waitForPollingStabilization,
  simulateControlCenterSetup
} = require('../helpers/ComponentTestHelpers');

const {
  validateControlOperationWorkflow,
  validateHTMXRequestResponse,
  validateConcurrentHTMXRequests,
  validateStatusPollingContinuity,
  validateLogPollingContinuity
} = require('../helpers/WorkflowValidators');

const testProjects = require('../fixtures/test-projects.json');
const expectedRequests = require('../fixtures/expected-requests.json');

test.describe('Control Operations Integration Workflow', () => {
  
  test.beforeEach(async ({ page }) => {
    // Setup control center environment
    await simulateControlCenterSetup(page);
    await setupTestProjectsInSidebar(page, testProjects.projects);
    
    // Wait for polling stabilization
    await waitForPollingStabilization(page, 2000);
  });
  
  test.describe('Build Operation Workflow', () => {
    
    test('should execute complete Build operation workflow', async ({ page }) => {
      const activeProject = testProjects.projects.find(p => p.isActive);
      const expectedFormData = expectedRequests.operations.build;
      
      // Render Build button and execute workflow
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      const success = await validateControlOperationWorkflow(page, 'Build', expectedFormData);
      expect(success).toBe(true);
    });
    
    test('should inject pre-request message for Build operation', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      // Monitor message injection
      const messagePromise = page.waitForFunction((expectedMessage) => {
        const outputContainer = document.querySelector('#ftl-output');
        return outputContainer && outputContainer.textContent.includes(expectedMessage);
      }, expectedFormData.message);
      
      const buildButton = page.locator('button:has-text("Build")');
      await buildButton.click();
      
      // Validate message injection
      await messagePromise;
      
      const outputText = await page.locator('#ftl-output').textContent();
      expect(outputText).toContain(expectedFormData.message);
      
      // Validate timestamp format
      expect(outputText).toMatch(/\[\d{2}:\d{2}:\d{2}\]/);
    });
    
    test('should send correct MCP request for Build operation', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      const expectedRequest = {
        endpoint: '/mcp',
        method: 'POST'
      };
      
      const buildButton = page.locator('button:has-text("Build")');
      
      const requestResponse = await validateHTMXRequestResponse(page, expectedRequest, async () => {
        await buildButton.click();
      });
      
      // Validate request body contains expected form data
      const requestBody = requestResponse.request.body;
      expect(requestBody).toContain(`method=${encodeURIComponent(expectedFormData.method)}`);
      expect(requestBody).toContain(`project_path=${encodeURIComponent(expectedFormData.project_path)}`);
      expect(requestBody).toContain(`message=${encodeURIComponent(expectedFormData.message)}`);
    });
    
    test('should handle Build operation response and update output', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      // Mock successful build response
      await page.route('/mcp', route => {
        route.fulfill({
          status: 200,
          contentType: 'text/html',
          body: `<div id="ftl-output">${expectedFormData.message}<br>Build completed successfully</div>`
        });
      });
      
      const buildButton = page.locator('button:has-text("Build")');
      await buildButton.click();
      
      // Wait for response to be processed
      await page.waitForTimeout(1000);
      
      const outputText = await page.locator('#ftl-output').textContent();
      expect(outputText).toContain('Build completed successfully');
    });
  });
  
  test.describe('Up Operation Workflow', () => {
    
    test('should execute complete Up operation workflow', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.up;
      
      await renderBaseButton(page, 'Up', 'submit', expectedFormData);
      
      const success = await validateControlOperationWorkflow(page, 'Up', expectedFormData);
      expect(success).toBe(true);
    });
    
    test('should handle Up operation with status change implications', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.up;
      
      await renderBaseButton(page, 'Up', 'submit', expectedFormData);
      
      // Track status polling during Up operation
      const pollingRequests = [];
      page.on('request', (req) => {
        if (req.url().includes('/htmx/status/poll')) {
          pollingRequests.push({
            timestamp: Date.now(),
            url: req.url()
          });
        }
      });
      
      const upButton = page.locator('button:has-text("Up")');
      await upButton.click();
      
      // Wait for operation and subsequent status polling
      await page.waitForTimeout(3000);
      
      // Should continue status polling after Up operation
      expect(pollingRequests.length).toBeGreaterThan(2);
    });
  });
  
  test.describe('Watch Operation Workflow', () => {
    
    test('should execute complete Watch operation workflow', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.watch;
      
      await renderBaseButton(page, 'Watch', 'submit', expectedFormData);
      
      const success = await validateControlOperationWorkflow(page, 'Watch', expectedFormData);
      expect(success).toBe(true);
    });
    
    test('should handle Watch operation with enhanced log polling', async ({ page }) => {
      const activeProject = testProjects.projects.find(p => p.isActive);
      const expectedFormData = expectedRequests.operations.watch;
      
      await renderBaseButton(page, 'Watch', 'submit', expectedFormData);
      
      const watchButton = page.locator('button:has-text("Watch")');
      await watchButton.click();
      
      // Wait for Watch operation to initiate
      await page.waitForTimeout(1000);
      
      // Validate enhanced log polling for Watch mode
      const logPollingStats = await validateLogPollingContinuity(page, activeProject.path, 4000);
      
      // Watch mode should have more frequent log polling
      expect(logPollingStats.totalRequests).toBeGreaterThan(6);
      expect(logPollingStats.avgInterval).toBeLessThan(600);
    });
  });
  
  test.describe('Cross-Operation Integration', () => {
    
    test('should handle sequential Build -> Up -> Watch workflow', async ({ page }) => {
      const operations = ['Build', 'Up', 'Watch'];
      const results = [];
      
      for (const operation of operations) {
        const expectedFormData = expectedRequests.operations[operation.toLowerCase()];
        
        await renderBaseButton(page, operation, 'submit', expectedFormData);
        
        const success = await validateControlOperationWorkflow(page, operation, expectedFormData);
        results.push(success);
        
        // Brief pause between operations
        await page.waitForTimeout(500);
      }
      
      // All operations should succeed
      expect(results.every(result => result === true)).toBe(true);
    });
    
    test('should maintain output continuity across multiple operations', async ({ page }) => {
      const operations = ['Build', 'Up'];
      
      for (const operation of operations) {
        const expectedFormData = expectedRequests.operations[operation.toLowerCase()];
        
        await renderBaseButton(page, operation, 'submit', expectedFormData);
        
        const button = page.locator(`button:has-text("${operation}")`);
        await button.click();
        
        // Wait for message to appear
        await page.waitForFunction((expectedMessage) => {
          const outputContainer = document.querySelector('#ftl-output');
          return outputContainer && outputContainer.textContent.includes(expectedMessage);
        }, expectedFormData.message);
      }
      
      // Output should contain messages from both operations
      const outputText = await page.locator('#ftl-output').textContent();
      expect(outputText).toContain(expectedRequests.operations.build.message);
      expect(outputText).toContain(expectedRequests.operations.up.message);
    });
    
    test('should handle concurrent control operations gracefully', async ({ page }) => {
      const operations = ['Build', 'Up'];
      
      // Setup buttons for concurrent operations
      for (const operation of operations) {
        const expectedFormData = expectedRequests.operations[operation.toLowerCase()];
        await renderBaseButton(page, operation, 'submit', expectedFormData);
      }
      
      // Define concurrent actions
      const concurrentActions = operations.map(operation => 
        async () => {
          const button = page.locator(`button:has-text("${operation}")`);
          await button.click();
        }
      );
      
      const results = await validateConcurrentHTMXRequests(page, concurrentActions, 10000);
      
      // Should handle concurrent operations without errors
      expect(results.totalErrors).toBe(0);
      expect(results.totalRequests).toBeGreaterThanOrEqual(operations.length);
    });
  });
  
  test.describe('Operation State and Polling Integration', () => {
    
    test('should maintain status polling during control operations', async ({ page }) => {
      const activeProject = testProjects.projects.find(p => p.isActive);
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      // Track status polling during operation
      const pollingRequests = [];
      page.on('request', (req) => {
        if (req.url().includes('/htmx/status/poll')) {
          pollingRequests.push({
            timestamp: Date.now(),
            url: req.url()
          });
        }
      });
      
      const buildButton = page.locator('button:has-text("Build")');
      await buildButton.click();
      
      // Continue monitoring polling after operation
      await page.waitForTimeout(4000);
      
      // Should maintain regular status polling
      expect(pollingRequests.length).toBeGreaterThan(3);
      
      // Validate polling includes current project
      const recentRequests = pollingRequests.slice(-2);
      for (const req of recentRequests) {
        const url = new URL(req.url, 'http://localhost');
        const path = url.searchParams.get('path');
        expect(path).toBe(activeProject.path);
      }
    });
    
    test('should coordinate log polling with Watch operation', async ({ page }) => {
      const activeProject = testProjects.projects.find(p => p.isActive);
      const expectedFormData = expectedRequests.operations.watch;
      
      await renderBaseButton(page, 'Watch', 'submit', expectedFormData);
      
      // Monitor log polling frequency before and after Watch
      const logPollingBefore = [];
      const logPollingAfter = [];
      
      // Monitor initial polling
      const beforeHandler = (req) => {
        if (req.url().includes('/htmx/logs/poll')) {
          logPollingBefore.push(Date.now());
        }
      };
      page.on('request', beforeHandler);
      
      await page.waitForTimeout(2000);
      page.off('request', beforeHandler);
      
      // Execute Watch operation
      const watchButton = page.locator('button:has-text("Watch")');
      await watchButton.click();
      
      // Monitor polling after Watch
      const afterHandler = (req) => {
        if (req.url().includes('/htmx/logs/poll')) {
          logPollingAfter.push(Date.now());
        }
      };
      page.on('request', afterHandler);
      
      await page.waitForTimeout(2000);
      page.off('request', afterHandler);
      
      // Watch mode should maintain or increase log polling frequency
      expect(logPollingAfter.length).toBeGreaterThanOrEqual(logPollingBefore.length);
    });
  });
  
  test.describe('Error Handling in Control Operations', () => {
    
    test('should handle MCP server error during Build operation', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      // Mock MCP server error
      await page.route('/mcp', route => {
        route.fulfill({
          status: 500,
          contentType: 'text/html',
          body: '<div class="error">MCP server error during build</div>'
        });
      });
      
      const buildButton = page.locator('button:has-text("Build")');
      await buildButton.click();
      
      // Should handle error gracefully
      await page.waitForTimeout(2000);
      
      // UI should remain functional
      await expect(buildButton).toBeEnabled();
      
      // Other operations should still be available
      const upButton = page.locator('button:has-text("Up")');
      if (await upButton.count() > 0) {
        await expect(upButton).toBeEnabled();
      }
    });
    
    test('should handle network timeout during control operations', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.up;
      
      await renderBaseButton(page, 'Up', 'submit', expectedFormData);
      
      // Mock network timeout (never fulfill)
      await page.route('/mcp', route => {
        // Don't fulfill to simulate timeout
      });
      
      const upButton = page.locator('button:has-text("Up")');
      await upButton.click();
      
      // Wait for potential timeout
      await page.waitForTimeout(5000);
      
      // Application should remain stable
      await expect(upButton).toBeEnabled();
      
      // Output container should still exist
      const outputContainer = page.locator('#ftl-output');
      await expect(outputContainer).toBeVisible();
    });
    
    test('should handle partial form data during operations', async ({ page }) => {
      // Render button with incomplete form data
      const incompleteFormData = {
        method: 'build',
        // Missing project_path and message
      };
      
      await renderBaseButton(page, 'Build', 'submit', incompleteFormData);
      
      const buildButton = page.locator('button:has-text("Build")');
      await buildButton.click();
      
      // Should handle incomplete data gracefully
      await page.waitForTimeout(2000);
      
      // Button should remain functional
      await expect(buildButton).toBeEnabled();
    });
  });
  
  test.describe('Performance and Responsiveness', () => {
    
    test('should maintain UI responsiveness during control operations', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      const buildButton = page.locator('button:has-text("Build")');
      
      // Start build operation
      const buildPromise = buildButton.click();
      
      // Test UI responsiveness during operation
      const responsiveTest = async () => {
        try {
          // Try to interact with other UI elements
          const sidebar = page.locator('#sidebar-projects');
          await sidebar.hover();
          
          // Try to hover over project buttons
          const projectButtons = page.locator('[data-project-name] button');
          const buttonCount = await projectButtons.count();
          
          if (buttonCount > 0) {
            await projectButtons.first().hover();
          }
          
          return true;
        } catch (e) {
          return false;
        }
      };
      
      const [buildResult, responsiveResult] = await Promise.all([
        buildPromise.then(() => true).catch(() => false),
        responsiveTest()
      ]);
      
      expect(responsiveResult).toBe(true);
    });
    
    test('should complete control operations within performance thresholds', async ({ page }) => {
      const expectedFormData = expectedRequests.operations.build;
      
      await renderBaseButton(page, 'Build', 'submit', expectedFormData);
      
      const buildButton = page.locator('button:has-text("Build")');
      
      // Measure operation completion time
      const startTime = Date.now();
      
      await buildButton.click();
      
      // Wait for pre-request message to appear (operation initiated)
      await page.waitForFunction((expectedMessage) => {
        const outputContainer = document.querySelector('#ftl-output');
        return outputContainer && outputContainer.textContent.includes(expectedMessage);
      }, expectedFormData.message);
      
      const initiationTime = Date.now() - startTime;
      
      // Operation initiation should be fast
      expect(initiationTime).toBeLessThan(1000); // 1 second max
      expect(initiationTime).toBeLessThan(500);  // 500ms preferred
    });
  });
});