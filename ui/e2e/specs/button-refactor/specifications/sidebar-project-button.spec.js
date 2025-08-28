/**
 * SidebarProjectButton Component Specification Tests
 * 
 * Tests the specification and desired behavior of the SidebarProjectButton component.
 * These tests verify the component meets design requirements for project switching,
 * status visualization, active/inactive states, and removal functionality.
 */

const { test, expect } = require('@playwright/test');
const { 
  renderSidebarProjectButton,
  setupTestProjectsInSidebar,
  validateProjectButtonCSS,
  measureHoverResponseTime,
  simulateProjectRemoval,
  hasVerticalTranslation
} = require('../helpers/ComponentTestHelpers');

const {
  validateProjectSwitchWorkflow,
  validateHTMXRequestResponse,
  validateOOBSwap
} = require('../helpers/WorkflowValidators');

const testProjects = require('../fixtures/test-projects.json');
const expectedRequests = require('../fixtures/expected-requests.json');
const uiColorScheme = require('../fixtures/ui-color-scheme.json');

test.describe('SidebarProjectButton Component Specification', () => {
  
  test.describe('HTML Structure and CSS Classes', () => {
    
    test('should render project item container with correct structure', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project (has isActive: false)
      
      await renderSidebarProjectButton(page, project);
      
      // Validate outer container structure
      const projectItem = page.locator(`[data-project-name="${project.name}"]`);
      await expect(projectItem).toHaveClass(/project-item/);
      await expect(projectItem).toHaveAttribute('data-project-path', project.path);
      
      // Validate inner structure - for inactive projects, button should have HTMX attributes
      const button = projectItem.locator('button[hx-post]');
      await expect(button).toBeVisible();
      
      const projectName = button.locator('.project-name');
      await expect(projectName).toContainText(project.name);
      
      const statusDot = button.locator('.status-dot');
      await expect(statusDot).toBeVisible();
    });
    
    test('should apply active project CSS classes correctly', async ({ page }) => {
      const project = testProjects.projects[0]; // active-project
      
      await renderSidebarProjectButton(page, project);
      
      const projectItem = page.locator('[data-project-name="active-project"]');
      await expect(projectItem).toHaveClass(/project-item-active/);
      await expect(projectItem).not.toHaveClass(/project-item-inactive/);
      
      const button = projectItem.locator('button');
      await expect(button).toHaveClass(new RegExp(uiColorScheme.cssClasses.baseButton));
    });
    
    test('should apply inactive project CSS classes correctly', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project
      
      await renderSidebarProjectButton(page, project);
      
      const projectItem = page.locator('[data-project-name="inactive-project"]');
      await expect(projectItem).toHaveClass(/project-item-inactive/);
      await expect(projectItem).not.toHaveClass(/project-item-active/);
    });
    
    test('should render status dot with correct color for each status', async ({ page }) => {
      for (const project of testProjects.projects) {
        await renderSidebarProjectButton(page, project);
        
        const statusDot = page.locator(`[data-project-name="${project.name}"] .status-dot`);
        const expectedColor = uiColorScheme.rgbColors.status_dots[project.status];
        
        await expect(statusDot).toHaveCSS('background-color', expectedColor);
      }
    });
  });
  
  test.describe('HTMX Integration', () => {
    
    test('should have correct HTMX attributes for project switching', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project
      
      await renderSidebarProjectButton(page, project);
      
      const button = page.locator(`[data-project-name="${project.name}"] button`);
      
      // Validate HTMX POST attributes
      await expect(button).toHaveAttribute('hx-post', '/htmx/project/switch');
      await expect(button).toHaveAttribute('hx-target', '#main-container');
      await expect(button).toHaveAttribute('hx-swap', 'outerHTML');
      await expect(button).toHaveAttribute('hx-vals', 
        JSON.stringify({ path: project.path }));
    });
    
    test('should include OOB swap for sidebar updates', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project
      
      await renderSidebarProjectButton(page, project);
      
      const button = page.locator(`[data-project-name="${project.name}"] button`);
      
      // Should include hx-include for sidebar update
      const hxInclude = await button.getAttribute('hx-include');
      expect(hxInclude).toContain('#sidebar-projects');
    });
    
    test('should not have HTMX attributes for active project', async ({ page }) => {
      const project = testProjects.projects[0]; // active-project
      
      await renderSidebarProjectButton(page, project);
      
      const button = page.locator(`[data-project-name="${project.name}"] button`);
      
      // Active project button should not have switching HTMX attributes
      await expect(button).not.toHaveAttribute('hx-post');
      await expect(button).not.toHaveAttribute('hx-vals');
    });
  });
  
  test.describe('Project Status Visualization', () => {
    
    test('should display running status with green dot', async ({ page }) => {
      const runningProject = testProjects.projects.find(p => p.status === 'running');
      
      await renderSidebarProjectButton(page, runningProject);
      
      const statusDot = page.locator(`[data-project-name="${runningProject.name}"] .status-dot`);
      await expect(statusDot).toHaveCSS('background-color', uiColorScheme.rgbColors.status_dots.running);
      
      // Should have running indicator class
      await expect(statusDot).toHaveClass(/status-running/);
    });
    
    test('should display watching status with blue dot', async ({ page }) => {
      const watchingProject = testProjects.projects.find(p => p.status === 'watching');
      
      await renderSidebarProjectButton(page, watchingProject);
      
      const statusDot = page.locator(`[data-project-name="${watchingProject.name}"] .status-dot`);
      await expect(statusDot).toHaveCSS('background-color', uiColorScheme.rgbColors.status_dots.watching);
      
      // Should have watching indicator class
      await expect(statusDot).toHaveClass(/status-watching/);
    });
    
    test('should display stopped status with gray dot', async ({ page }) => {
      const stoppedProject = testProjects.projects.find(p => p.status === 'stopped');
      
      await renderSidebarProjectButton(page, stoppedProject);
      
      const statusDot = page.locator(`[data-project-name="${stoppedProject.name}"] .status-dot`);
      await expect(statusDot).toHaveCSS('background-color', uiColorScheme.rgbColors.status_dots.stopped);
      
      // Should have stopped indicator class
      await expect(statusDot).toHaveClass(/status-stopped/);
    });
    
    test('should update status dot when project status changes', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project initially stopped
      
      await renderSidebarProjectButton(page, project);
      
      // Initial state - stopped
      let statusDot = page.locator(`[data-project-name="${project.name}"] .status-dot`);
      await expect(statusDot).toHaveCSS('background-color', uiColorScheme.rgbColors.status_dots.stopped);
      
      // Simulate status change to running via re-render
      const runningProject = { ...project, status: 'running' };
      await renderSidebarProjectButton(page, runningProject);
      
      // Status should now be running
      statusDot = page.locator(`[data-project-name="${project.name}"] .status-dot`);
      await expect(statusDot).toHaveCSS('background-color', uiColorScheme.rgbColors.status_dots.running);
    });
  });
  
  test.describe('Hover Behavior', () => {
    
    test('should implement unified hover behavior matching BaseButton', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project (has hover)
      
      await renderSidebarProjectButton(page, project);
      
      const button = page.locator(`[data-project-name="${project.name}"] button`);
      
      // Get initial styles
      const initialBgColor = await button.evaluate(el => getComputedStyle(el).backgroundColor);
      const initialBorderColor = await button.evaluate(el => getComputedStyle(el).borderColor);
      const initialTransform = await button.evaluate(el => getComputedStyle(el).transform);
      
      // Hover over button
      await button.hover();
      
      // Validate hover state changes
      await expect(button).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg_hover);
      await expect(button).toHaveCSS('border-color', uiColorScheme.rgbColors.border_hover);
      
      // Validate hover transform using helper function
      const hoverTransform = await button.evaluate(el => getComputedStyle(el).transform);
      expect(hasVerticalTranslation(hoverTransform, -1)).toBe(true);
      
      // Mouse out
      await page.mouse.move(0, 0);
      
      // Validate return to normal state
      await expect(button).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
      await expect(button).toHaveCSS('border-color', uiColorScheme.rgbColors.button_border);
      
      // Validate normal transform using helper function
      const normalTransform = await button.evaluate(el => getComputedStyle(el).transform);
      expect(hasVerticalTranslation(normalTransform, 0)).toBe(true);
    });
    
    test('should not have hover behavior for active project', async ({ page }) => {
      const project = testProjects.projects[0]; // active-project
      
      await renderSidebarProjectButton(page, project);
      
      const button = page.locator(`[data-project-name="${project.name}"] button`);
      
      // Get initial styles
      const initialBgColor = await button.evaluate(el => getComputedStyle(el).backgroundColor);
      const initialTransform = await button.evaluate(el => getComputedStyle(el).transform);
      
      // Hover over button
      await button.hover();
      
      // Active project button should not change on hover
      const hoverBgColor = await button.evaluate(el => getComputedStyle(el).backgroundColor);
      const hoverTransform = await button.evaluate(el => getComputedStyle(el).transform);
      
      expect(hoverBgColor).toBe(initialBgColor);
      expect(hoverTransform).toBe(initialTransform);
    });
    
  });
  
  
  test.describe('Multiple Project Management', () => {
    
    
    test('should maintain only one active project', async ({ page }) => {
      await setupTestProjectsInSidebar(page, testProjects.projects);
      
      // Count active projects
      const activeProjects = page.locator('.project-item-active');
      await expect(activeProjects).toHaveCount(1);
      
      // Validate the correct project is active
      const activeProject = activeProjects.first();
      const activeName = activeProject.locator('.project-name');
      await expect(activeName).toContainText('active-project');
    });
    
  });
  
  test.describe('Project Removal Functionality', () => {
    
    
    test('should not show remove button for active project', async ({ page }) => {
      const project = testProjects.projects[0]; // active-project
      
      await renderSidebarProjectButton(page, { project, includeRemoveButton: true });
      
      // Active project should not have remove button
      const removeButton = page.locator(`[data-project-name="${project.name}"] .remove-project-btn`);
      await expect(removeButton).not.toBeVisible();
    });
    
    test('should confirm removal before executing', async ({ page }) => {
      const project = testProjects.projects[1]; // inactive-project
      
      await renderSidebarProjectButton(page, { project, includeRemoveButton: true });
      
      const removeButton = page.locator(`[data-project-name="${project.name}"] .remove-project-btn`);
      
      // Mock dialog confirmation
      page.on('dialog', dialog => {
        expect(dialog.message()).toContain(`Remove project "${project.name}"`);
        dialog.accept();
      });
      
      await removeButton.click();
    });
  });
  
  
});