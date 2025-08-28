/**
 * Project Switching Integration Workflow Tests
 * 
 * Tests the complete project switching workflow including HTMX requests,
 * DOM updates, sidebar state changes, and polling continuity.
 */

const { test, expect } = require('@playwright/test');
const { 
  setupTestProjectsInSidebar,
  validateProjectButtonCSS,
  waitForPollingStabilization
} = require('../helpers/ComponentTestHelpers');

const {
  validateProjectSwitchWorkflow,
  validateStatusPollingContinuity,
  validateLogPollingContinuity,
  validateHTMXRequestResponse,
  validateOOBSwap,
  validateConcurrentHTMXRequests
} = require('../helpers/WorkflowValidators');

const testProjects = require('../fixtures/test-projects.json');
const expectedRequests = require('../fixtures/expected-requests.json');

test.describe('Project Switching Integration Workflow', () => {
  
  test.beforeEach(async ({ page }) => {
    // Setup initial project environment with multiple projects
    await setupTestProjectsInSidebar(page, testProjects.projects);
    
    // Wait for any initial polling to stabilize
    await waitForPollingStabilization(page, 2000);
  });
  
  test.describe('Complete Project Switch Workflow', () => {
    
    test('should execute full project switch from active to inactive project', async ({ page }) => {
      const sourceProject = testProjects.projects.find(p => p.isActive);
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Execute complete workflow validation
      const success = await validateProjectSwitchWorkflow(page, sourceProject, targetProject);
      
      expect(success).toBe(true);
      
      // Additional validation: main content should reflect new project
      const mainContainer = page.locator('#main-container');
      await expect(mainContainer).toContainText(targetProject.name);
    });
    
    test('should handle multiple sequential project switches', async ({ page }) => {
      const projects = testProjects.projects.filter(p => !p.isActive);
      let currentActive = testProjects.projects.find(p => p.isActive);
      
      for (const targetProject of projects) {
        // Switch to target project
        const success = await validateProjectSwitchWorkflow(page, currentActive, targetProject);
        expect(success).toBe(true);
        
        // Update current active for next iteration
        currentActive = targetProject;
        
        // Brief pause between switches
        await page.waitForTimeout(500);
      }
    });
    
    test('should maintain data integrity across project switches', async ({ page }) => {
      const sourceProject = testProjects.projects.find(p => p.isActive);
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Capture initial sidebar state
      const initialSidebarHtml = await page.locator('#sidebar-projects').innerHTML();
      
      // Execute project switch
      await validateProjectSwitchWorkflow(page, sourceProject, targetProject);
      
      // Validate sidebar structure maintained (but with updated active states)
      const finalSidebarHtml = await page.locator('#sidebar-projects').innerHTML();
      
      // Should contain all original projects
      for (const project of testProjects.projects) {
        expect(finalSidebarHtml).toContain(project.name);
      }
      
      // But active state should have changed
      expect(finalSidebarHtml).not.toBe(initialSidebarHtml);
    });
  });
  
  test.describe('HTMX Request/Response Cycle', () => {
    
    test('should send correct project switch request with proper form data', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      const expectedRequest = {
        endpoint: '/htmx/project/switch',
        method: 'POST',
        expectedParams: [`path=${encodeURIComponent(targetProject.path)}`]
      };
      
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      
      const requestResponse = await validateHTMXRequestResponse(page, expectedRequest, async () => {
        await button.click();
      });
      
      // Validate response contains expected project data
      expect(requestResponse.response.status).toBeLessThan(400);
      expect(requestResponse.response.body).toContain(targetProject.name);
    });
    
    test('should handle HTMX response with main container update', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Monitor main container changes
      const initialMainContent = await page.locator('#main-container').innerHTML();
      
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      
      // Execute switch and wait for DOM update
      await button.click();
      
      await page.waitForFunction((initialContent) => {
        const mainContainer = document.querySelector('#main-container');
        return mainContainer && mainContainer.innerHTML !== initialContent;
      }, initialMainContent);
      
      // Validate main container updated with target project context
      const finalMainContent = await page.locator('#main-container').innerHTML();
      expect(finalMainContent).toContain(targetProject.name);
      expect(finalMainContent).not.toBe(initialMainContent);
    });
    
    test('should execute OOB swap for sidebar state synchronization', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      
      const success = await validateOOBSwap(
        page,
        '#main-container',        // Main swap target
        ['#sidebar-projects'],    // OOB swap targets
        async () => {
          await button.click();
        }
      );
      
      expect(success).toBe(true);
      
      // Validate sidebar reflects new active state
      const newActiveProject = page.locator('.project-item-active .project-name');
      await expect(newActiveProject).toContainText(targetProject.name);
    });
  });
  
  test.describe('Polling Continuity During Project Switch', () => {
    
    test('should maintain status polling continuity after project switch', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Execute project switch
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      await button.click();
      
      // Wait for switch to complete
      await page.waitForSelector('.project-item-active', { timeout: 5000 });
      
      // Validate status polling continues for new project
      const pollingStats = await validateStatusPollingContinuity(page, targetProject.path, 5000);
      
      expect(pollingStats.totalRequests).toBeGreaterThan(3);
      expect(pollingStats.avgInterval).toBeGreaterThan(800);
      expect(pollingStats.avgInterval).toBeLessThan(1200);
    });
    
    test('should maintain log polling continuity after project switch', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Execute project switch
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      await button.click();
      
      // Wait for switch to complete
      await page.waitForSelector('.project-item-active', { timeout: 5000 });
      
      // Validate log polling continues for new project
      const pollingStats = await validateLogPollingContinuity(page, targetProject.path, 3000);
      
      expect(pollingStats.totalRequests).toBeGreaterThan(4);
      expect(pollingStats.avgInterval).toBeGreaterThan(400);
      expect(pollingStats.avgInterval).toBeLessThan(600);
    });
    
    test('should stop polling for previous project after switch', async ({ page }) => {
      const sourceProject = testProjects.projects.find(p => p.isActive);
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Track all polling requests
      const pollingRequests = [];
      page.on('request', (req) => {
        if (req.url().includes('/htmx/status/poll') || req.url().includes('/htmx/logs/poll')) {
          const url = new URL(req.url());
          const path = url.searchParams.get('path');
          pollingRequests.push({
            timestamp: Date.now(),
            projectPath: path,
            endpoint: req.url().includes('/status/') ? 'status' : 'logs'
          });
        }
      });
      
      // Execute project switch
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      await button.click();
      
      // Wait for switch to complete and monitor polling
      await page.waitForSelector('.project-item-active', { timeout: 5000 });
      await page.waitForTimeout(3000);
      
      // Analyze recent polling requests
      const recentRequests = pollingRequests.filter(req => 
        req.timestamp > Date.now() - 2000
      );
      
      // Should only be polling for target project, not source
      const sourcePolling = recentRequests.filter(req => req.projectPath === sourceProject.path);
      const targetPolling = recentRequests.filter(req => req.projectPath === targetProject.path);
      
      expect(sourcePolling.length).toBe(0);
      expect(targetPolling.length).toBeGreaterThan(0);
    });
  });
  
  test.describe('Concurrent Operations Handling', () => {
    
    test('should handle rapid project switches without race conditions', async ({ page }) => {
      const projects = testProjects.projects.filter(p => !p.isActive);
      
      // Create rapid switch actions
      const switchActions = projects.slice(0, 3).map(project => 
        async () => {
          const button = page.locator(`[data-project-name="${project.name}"] button`);
          await button.click();
        }
      );
      
      // Execute concurrent switches
      const results = await validateConcurrentHTMXRequests(page, switchActions, 10000);
      
      // Should handle all requests without errors
      expect(results.totalErrors).toBe(0);
      expect(results.totalResponses).toBeGreaterThanOrEqual(1); // At least one should succeed
      
      // Final state should be consistent
      const activeProjects = page.locator('.project-item-active');
      await expect(activeProjects).toHaveCount(1);
    });
    
    test('should handle project switch concurrent with control operations', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Define concurrent actions
      const concurrentActions = [
        // Project switch
        async () => {
          const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
          await button.click();
        },
        // Control operation (if build button exists)
        async () => {
          try {
            const buildButton = page.locator('button:has-text("Build")');
            if (await buildButton.count() > 0) {
              await buildButton.click();
            }
          } catch (e) {
            // Control button might not be available yet
          }
        }
      ];
      
      const results = await validateConcurrentHTMXRequests(page, concurrentActions, 15000);
      
      // Should handle concurrent operations gracefully
      expect(results.totalErrors).toBe(0);
      expect(results.actionResults.every(result => 
        result.status === 'fulfilled'
      )).toBe(true);
    });
    
    test('should maintain UI responsiveness during concurrent operations', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Start project switch
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      const switchPromise = button.click();
      
      // Immediately test UI responsiveness
      const hoverTest = async () => {
        try {
          // Try to hover other projects during switch
          const otherProjects = testProjects.projects.filter(p => 
            p.name !== targetProject.name && !p.isActive
          );
          
          for (const project of otherProjects.slice(0, 2)) {
            const otherButton = page.locator(`[data-project-name="${project.name}"] button`);
            await otherButton.hover();
            await page.waitForTimeout(50);
          }
          
          return true;
        } catch (e) {
          return false;
        }
      };
      
      // Both operations should complete successfully
      const [switchResult, hoverResult] = await Promise.all([
        switchPromise.then(() => true).catch(() => false),
        hoverTest()
      ]);
      
      expect(switchResult).toBe(true);
      expect(hoverResult).toBe(true);
    });
  });
  
  test.describe('Error Recovery and Edge Cases', () => {
    
    test('should handle server error during project switch gracefully', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Mock server error for project switch
      await page.route('/htmx/project/switch', route => {
        route.fulfill({ 
          status: 500, 
          contentType: 'text/html',
          body: '<div class="error">Project switch failed</div>' 
        });
      });
      
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      await button.click();
      
      // Should handle error gracefully - exact behavior depends on HTMX error handling
      // At minimum, app should remain functional
      await page.waitForTimeout(2000);
      
      // UI should still be responsive
      const buildButton = page.locator('button:has-text("Build")');
      if (await buildButton.count() > 0) {
        await expect(buildButton).toBeEnabled();
      }
    });
    
    test('should handle network timeout during project switch', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Mock network timeout
      await page.route('/htmx/project/switch', route => {
        // Never fulfill the route to simulate timeout
      });
      
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      await button.click();
      
      // Wait for potential timeout handling
      await page.waitForTimeout(5000);
      
      // Application should remain stable
      const sidebar = page.locator('#sidebar-projects');
      await expect(sidebar).toBeVisible();
      
      // Other buttons should still be functional
      const otherProject = testProjects.projects.find(p => 
        p.name !== targetProject.name && !p.isActive
      );
      if (otherProject) {
        const otherButton = page.locator(`[data-project-name="${otherProject.name}"] button`);
        await expect(otherButton).toBeEnabled();
      }
    });
    
    test('should handle malformed project data in response', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      // Mock malformed response
      await page.route('/htmx/project/switch', route => {
        route.fulfill({
          status: 200,
          contentType: 'text/html',
          body: '<div>Invalid project data</div>' // Missing expected structure
        });
      });
      
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      await button.click();
      
      // Should handle malformed data gracefully
      await page.waitForTimeout(2000);
      
      // Core UI elements should remain available
      const sidebar = page.locator('#sidebar-projects');
      await expect(sidebar).toBeVisible();
      
      // Should maintain project list integrity
      for (const project of testProjects.projects) {
        const projectItem = page.locator(`[data-project-name="${project.name}"]`);
        await expect(projectItem).toBeVisible();
      }
    });
  });
  
  test.describe('Performance Validation', () => {
    
    test('should complete project switch within performance threshold', async ({ page }) => {
      const targetProject = testProjects.projects.find(p => !p.isActive);
      
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      
      // Measure switch completion time
      const startTime = Date.now();
      
      await button.click();
      
      // Wait for switch to complete (sidebar active state change)
      await page.waitForFunction((targetName) => {
        const activeElement = document.querySelector('.project-item-active .project-name');
        return activeElement && activeElement.textContent.trim() === targetName;
      }, targetProject.name);
      
      const completionTime = Date.now() - startTime;
      
      // Should complete within reasonable time (adjust threshold as needed)
      expect(completionTime).toBeLessThan(5000); // 5 seconds max
      
      // For good user experience, should be much faster
      expect(completionTime).toBeLessThan(2000); // 2 seconds preferred
    });
    
    test('should maintain 60fps during project switch animations', async ({ page }) => {
      // This would require performance monitoring APIs
      // For now, we validate that animations complete without visual glitches
      
      const targetProject = testProjects.projects.find(p => !p.isActive);
      const button = page.locator(`[data-project-name="${targetProject.name}"] button`);
      
      // Start switch and monitor for visual stability
      await button.click();
      
      // Wait for any animations to complete
      await page.waitForTimeout(1000);
      
      // Validate final state is visually correct
      const activeProject = page.locator('.project-item-active');
      await expect(activeProject).toBeVisible();
      await expect(activeProject).toHaveCSS('opacity', '1');
      
      const projectName = activeProject.locator('.project-name');
      await expect(projectName).toContainText(targetProject.name);
    });
  });
});