const { test, expect } = require('@playwright/test');
const TestDataManager = require('../../../../e2e-tests/e2e/data/TestDataManager');

test.describe('Control Center Buttons', () => {
  test.beforeEach(async ({ page }) => {
    // Initialize test data with a single project for control center testing
    await TestDataManager.initialize('single');
    await page.goto('/');
    await page.waitForSelector('[data-testid="control-center"]', { timeout: 10000 });
  });

  test('Build button has correct hover behavior and click functionality', async ({ page }) => {
    const buildButton = page.locator('button:has-text("Build")');
    
    // Check initial state
    await expect(buildButton).toBeVisible();
    
    // Test hover color change
    await buildButton.hover();
    await expect(buildButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
    
    // Test hover out
    await page.mouse.move(0, 0); // Move mouse away
    await expect(buildButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    
    // Test click sends build command
    await buildButton.click();
    await expect(page.locator('#ftl-output')).toContainText('Sending build command');
  });

  test('Up button has correct hover behavior and click functionality', async ({ page }) => {
    const upButton = page.locator('button:has-text("Up")');
    
    // Check initial state
    await expect(upButton).toBeVisible();
    
    // Test hover transform
    await upButton.hover();
    await expect(upButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
    
    // Test hover out
    await page.mouse.move(0, 0);
    await expect(upButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    
    // Test click sends up command
    await upButton.click();
    await expect(page.locator('#ftl-output')).toContainText('Starting FTL server');
  });

  test('Watch button has correct hover behavior and click functionality', async ({ page }) => {
    const watchButton = page.locator('button:has-text("Watch")');
    
    // Check initial state
    await expect(watchButton).toBeVisible();
    
    // Test hover transform
    await watchButton.hover();
    await expect(watchButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
    
    // Test hover out
    await page.mouse.move(0, 0);
    await expect(watchButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    
    // Test click sends watch command
    await watchButton.click();
    await expect(page.locator('#ftl-output')).toContainText('Starting FTL in watch mode');
  });

  test('All control center buttons use consistent hover colors', async ({ page }) => {
    const buttons = [
      page.locator('button:has-text("Build")'),
      page.locator('button:has-text("Up")'),
      page.locator('button:has-text("Watch")')
    ];

    for (const button of buttons) {
      // Test that all buttons have consistent hover behavior
      await button.hover();
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
      
      // Move away to reset
      await page.mouse.move(0, 0);
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    }
  });
});