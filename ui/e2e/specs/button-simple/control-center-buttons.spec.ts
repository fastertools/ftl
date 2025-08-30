import { test, expect, Page } from '@playwright/test';
import TestHelpers from '../../utils/test-helpers';

test.describe('Control Center Buttons', () => {
  test.beforeEach(async ({ page }: { page: Page }) => {
    // Initialize test environment with a single project for control center testing
    const projectSetup = await TestHelpers.setupSingleProject();
    await page.goto(projectSetup.baseUrl);
    
    // Use enhanced HTMX waiting instead of waitForSelector
    await TestHelpers.waitForHTMX(page, 10000);
    await page.waitForSelector('[data-testid="control-center"]', { timeout: 10000 });
  });

  test('Build button has correct hover behavior and click functionality', async ({ page }: { page: Page }) => {
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

  test('Up button has correct hover behavior and click functionality', async ({ page }: { page: Page }) => {
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

  test('Watch button has correct hover behavior and click functionality', async ({ page }: { page: Page }) => {
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

  test('All control center buttons use consistent hover colors', async ({ page }: { page: Page }) => {
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