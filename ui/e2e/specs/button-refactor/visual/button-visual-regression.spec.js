/**
 * Button Visual Regression Tests
 * 
 * Tests visual appearance and consistency of BaseButton and SidebarProjectButton
 * components using screenshot comparison and visual validation.
 */

const { test, expect } = require('@playwright/test');
const { 
  renderBaseButton,
  renderSidebarProjectButton,
  setupTestProjectsInSidebar,
  simulateControlCenterSetup,
  captureButtonScreenshot,
  setupVisualTestEnvironment
} = require('../helpers/ComponentTestHelpers');

const testProjects = require('../fixtures/test-projects.json');
const uiColorScheme = require('../fixtures/ui-color-scheme.json');

// Configure visual testing options
const visualTestOptions = {
  threshold: 0.1,        // 10% difference threshold
  animations: 'disabled', // Disable animations for consistent screenshots
  clip: null,           // Will be set per test
  fullPage: false
};

test.describe('Button Visual Regression Tests', () => {
  
  test.beforeEach(async ({ page }) => {
    // Setup visual test environment with consistent styling
    await setupVisualTestEnvironment(page);
    
    // Disable animations for consistent screenshots
    await page.addStyleTag({
      content: `
        *, *::before, *::after {
          animation-duration: 0s !important;
          animation-delay: 0s !important;
          transition-duration: 0s !important;
          transition-delay: 0s !important;
        }
      `
    });
  });
  
  test.describe('BaseButton Visual Validation', () => {
    
    test('should render BaseButton in normal state consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      
      // Capture screenshot of Build button in normal state
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-build-normal');
      
      // Compare with baseline (will create baseline on first run)
      await expect(screenshot).toMatchSnapshot('base-button-build-normal.png', visualTestOptions);
    });
    
    test('should render BaseButton in hover state consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      
      // Hover over button to trigger hover state
      await button.hover();
      
      // Wait for hover transition to complete
      await page.waitForTimeout(250);
      
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-build-hover');
      
      await expect(screenshot).toMatchSnapshot('base-button-build-hover.png', visualTestOptions);
    });
    
    test('should render all BaseButton variations consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      
      const buttonTypes = ['Build', 'Up', 'Watch'];
      
      for (const buttonType of buttonTypes) {
        await renderBaseButton(page, buttonType, 'submit', {});
        
        const button = page.locator(`button:has-text("${buttonType}")`);
        const screenshot = await captureButtonScreenshot(page, button, `base-button-${buttonType.toLowerCase()}-normal`);
        
        await expect(screenshot).toMatchSnapshot(`base-button-${buttonType.toLowerCase()}-normal.png`, visualTestOptions);
      }
    });
    
    test('should render BaseButton focus state consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      
      // Focus the button
      await button.focus();
      
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-build-focus');
      
      await expect(screenshot).toMatchSnapshot('base-button-build-focus.png', visualTestOptions);
    });
  });
  
  test.describe('SidebarProjectButton Visual Validation', () => {
    
    test('should render inactive SidebarProjectButton consistently', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderSidebarProjectButton(page, inactiveProject);
      
      const projectItem = page.locator(`[data-project-name="${inactiveProject.name}"]`);
      
      const screenshot = await captureButtonScreenshot(page, projectItem, 'sidebar-project-button-inactive');
      
      await expect(screenshot).toMatchSnapshot('sidebar-project-button-inactive.png', visualTestOptions);
    });
    
    test('should render active SidebarProjectButton consistently', async ({ page }) => {
      const activeProject = testProjects.projects.find(p => p.isActive);
      
      await renderSidebarProjectButton(page, activeProject);
      
      const projectItem = page.locator(`[data-project-name="${activeProject.name}"]`);
      
      const screenshot = await captureButtonScreenshot(page, projectItem, 'sidebar-project-button-active');
      
      await expect(screenshot).toMatchSnapshot('sidebar-project-button-active.png', visualTestOptions);
    });
    
    test('should render SidebarProjectButton with different status dots', async ({ page }) => {
      const statusTypes = ['running', 'watching', 'stopped'];
      
      for (const status of statusTypes) {
        const project = testProjects.projects.find(p => p.status === status) || {
          name: `test-${status}-project`,
          path: `/path/to/${status}-project`,
          status: status,
          isActive: false
        };
        
        await renderSidebarProjectButton(page, project);
        
        const projectItem = page.locator(`[data-project-name="${project.name}"]`);
        const screenshot = await captureButtonScreenshot(page, projectItem, `sidebar-project-button-${status}`);
        
        await expect(screenshot).toMatchSnapshot(`sidebar-project-button-${status}.png`, visualTestOptions);
      }
    });
    
    test('should render SidebarProjectButton hover state consistently', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderSidebarProjectButton(page, inactiveProject);
      
      const button = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Hover over button
      await button.hover();
      await page.waitForTimeout(250);
      
      const projectItem = page.locator(`[data-project-name="${inactiveProject.name}"]`);
      const screenshot = await captureButtonScreenshot(page, projectItem, 'sidebar-project-button-hover');
      
      await expect(screenshot).toMatchSnapshot('sidebar-project-button-hover.png', visualTestOptions);
    });
  });
  
  test.describe('Multi-Component Layout Validation', () => {
    
    test('should render complete sidebar with multiple projects consistently', async ({ page }) => {
      await setupTestProjectsInSidebar(page, testProjects.projects);
      
      const sidebar = page.locator('#sidebar-projects');
      
      // Capture full sidebar layout
      const screenshot = await page.screenshot({
        clip: await sidebar.boundingBox(),
        animations: 'disabled'
      });
      
      await expect(screenshot).toMatchSnapshot('sidebar-complete-layout.png', visualTestOptions);
    });
    
    test('should render control center with all buttons consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      
      // Render all control buttons
      const buttonTypes = ['Build', 'Up', 'Watch'];
      for (const buttonType of buttonTypes) {
        await renderBaseButton(page, buttonType, 'submit', {});
      }
      
      const controlCenter = page.locator('#control-center, .control-center, [data-testid="control-center"]').first();
      
      // Fallback to a broader selector if control center not found
      if (await controlCenter.count() === 0) {
        // Capture the area containing all buttons
        const allButtons = page.locator('button:has-text("Build"), button:has-text("Up"), button:has-text("Watch")');
        const firstButton = allButtons.first();
        const lastButton = allButtons.last();
        
        if (await firstButton.count() > 0 && await lastButton.count() > 0) {
          const firstBox = await firstButton.boundingBox();
          const lastBox = await lastButton.boundingBox();
          
          const combinedBox = {
            x: Math.min(firstBox.x, lastBox.x),
            y: Math.min(firstBox.y, lastBox.y),
            width: Math.max(firstBox.x + firstBox.width, lastBox.x + lastBox.width) - Math.min(firstBox.x, lastBox.x),
            height: Math.max(firstBox.y + firstBox.height, lastBox.y + lastBox.height) - Math.min(firstBox.y, lastBox.y)
          };
          
          const screenshot = await page.screenshot({
            clip: combinedBox,
            animations: 'disabled'
          });
          
          await expect(screenshot).toMatchSnapshot('control-center-complete-layout.png', visualTestOptions);
        }
      } else {
        const screenshot = await page.screenshot({
          clip: await controlCenter.boundingBox(),
          animations: 'disabled'
        });
        
        await expect(screenshot).toMatchSnapshot('control-center-complete-layout.png', visualTestOptions);
      }
    });
    
    test('should render integrated interface (sidebar + control center) consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await setupTestProjectsInSidebar(page, testProjects.projects);
      
      // Render control center buttons
      const buttonTypes = ['Build', 'Up', 'Watch'];
      for (const buttonType of buttonTypes) {
        await renderBaseButton(page, buttonType, 'submit', {});
      }
      
      // Wait for all components to be rendered
      await page.waitForTimeout(500);
      
      // Capture full interface
      const screenshot = await page.screenshot({
        fullPage: true,
        animations: 'disabled'
      });
      
      await expect(screenshot).toMatchSnapshot('integrated-interface-layout.png', {
        ...visualTestOptions,
        fullPage: true
      });
    });
  });
  
  test.describe('Cross-Browser Visual Consistency', () => {
    
    test('should render buttons consistently in different browser engines', async ({ page, browserName }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      
      // Capture screenshot with browser name in filename
      const screenshot = await captureButtonScreenshot(page, button, `base-button-${browserName}`);
      
      // Compare with browser-specific baseline
      await expect(screenshot).toMatchSnapshot(`base-button-${browserName}.png`, visualTestOptions);
    });
    
    test('should render sidebar projects consistently across browsers', async ({ page, browserName }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderSidebarProjectButton(page, inactiveProject);
      
      const projectItem = page.locator(`[data-project-name="${inactiveProject.name}"]`);
      const screenshot = await captureButtonScreenshot(page, projectItem, `sidebar-project-${browserName}`);
      
      await expect(screenshot).toMatchSnapshot(`sidebar-project-${browserName}.png`, visualTestOptions);
    });
  });
  
  test.describe('Responsive Visual Validation', () => {
    
    test('should render buttons consistently at desktop resolution', async ({ page }) => {
      await page.setViewportSize({ width: 1920, height: 1080 });
      
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-desktop');
      
      await expect(screenshot).toMatchSnapshot('base-button-desktop.png', visualTestOptions);
    });
    
    test('should render buttons consistently at tablet resolution', async ({ page }) => {
      await page.setViewportSize({ width: 1024, height: 768 });
      
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-tablet');
      
      await expect(screenshot).toMatchSnapshot('base-button-tablet.png', visualTestOptions);
    });
    
    test('should render buttons consistently at mobile resolution', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-mobile');
      
      await expect(screenshot).toMatchSnapshot('base-button-mobile.png', visualTestOptions);
    });
    
    test('should render sidebar consistently at different resolutions', async ({ page }) => {
      const resolutions = [
        { name: 'desktop', width: 1920, height: 1080 },
        { name: 'tablet', width: 1024, height: 768 },
        { name: 'mobile', width: 375, height: 667 }
      ];
      
      for (const resolution of resolutions) {
        await page.setViewportSize({ width: resolution.width, height: resolution.height });
        
        await setupTestProjectsInSidebar(page, testProjects.projects);
        
        const sidebar = page.locator('#sidebar-projects');
        
        if (await sidebar.count() > 0) {
          const screenshot = await page.screenshot({
            clip: await sidebar.boundingBox(),
            animations: 'disabled'
          });
          
          await expect(screenshot).toMatchSnapshot(`sidebar-${resolution.name}.png`, visualTestOptions);
        }
      }
    });
  });
  
  test.describe('State Transition Visual Validation', () => {
    
    test('should visually validate hover transition smoothness', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {});
      
      const button = page.locator('button:has-text("Build")');
      
      // Capture normal state
      const normalScreenshot = await captureButtonScreenshot(page, button, 'transition-normal');
      await expect(normalScreenshot).toMatchSnapshot('transition-normal.png', visualTestOptions);
      
      // Hover and capture mid-transition (if possible)
      await button.hover();
      await page.waitForTimeout(100); // Mid-transition
      
      const midTransitionScreenshot = await captureButtonScreenshot(page, button, 'transition-mid');
      await expect(midTransitionScreenshot).toMatchSnapshot('transition-mid.png', visualTestOptions);
      
      // Capture final hover state
      await page.waitForTimeout(250); // Full transition
      
      const hoverScreenshot = await captureButtonScreenshot(page, button, 'transition-hover');
      await expect(hoverScreenshot).toMatchSnapshot('transition-hover.png', visualTestOptions);
    });
    
    test('should visually validate project switch state changes', async ({ page }) => {
      await setupTestProjectsInSidebar(page, testProjects.projects);
      
      const activeProject = testProjects.projects.find(p => p.isActive);
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      // Capture initial state
      const initialScreenshot = await page.screenshot({
        clip: await page.locator('#sidebar-projects').boundingBox(),
        animations: 'disabled'
      });
      
      await expect(initialScreenshot).toMatchSnapshot('project-switch-initial.png', visualTestOptions);
      
      // Simulate project switch (would typically involve clicking and waiting for response)
      // For visual testing, we can simulate the state change
      const inactiveButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Hover to show switch intent
      await inactiveButton.hover();
      await page.waitForTimeout(250);
      
      const switchIntentScreenshot = await page.screenshot({
        clip: await page.locator('#sidebar-projects').boundingBox(),
        animations: 'disabled'
      });
      
      await expect(switchIntentScreenshot).toMatchSnapshot('project-switch-intent.png', visualTestOptions);
    });
  });
  
  test.describe('Error State Visual Validation', () => {
    
    test('should render disabled button states consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {}, { disabled: true });
      
      const button = page.locator('button:has-text("Build")');
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-disabled');
      
      await expect(screenshot).toMatchSnapshot('base-button-disabled.png', visualTestOptions);
    });
    
    test('should render error state buttons consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {}, { error: true });
      
      const button = page.locator('button:has-text("Build")');
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-error');
      
      await expect(screenshot).toMatchSnapshot('base-button-error.png', visualTestOptions);
    });
    
    test('should render loading state buttons consistently', async ({ page }) => {
      await simulateControlCenterSetup(page);
      await renderBaseButton(page, 'Build', 'submit', {}, { loading: true });
      
      const button = page.locator('button:has-text("Build")');
      const screenshot = await captureButtonScreenshot(page, button, 'base-button-loading');
      
      await expect(screenshot).toMatchSnapshot('base-button-loading.png', visualTestOptions);
    });
  });
});