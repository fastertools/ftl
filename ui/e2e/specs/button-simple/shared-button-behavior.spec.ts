import { test, expect, Page, Locator } from '@playwright/test';
import TestHelpers from '../../utils/test-helpers';

test.describe('Shared Button Behavior', () => {
  test.beforeEach(async ({ page }: { page: Page }) => {
    // Initialize test environment with multiple projects for sidebar testing
    const projectSetup = await TestHelpers.setupMultipleProjects();
    await page.goto(projectSetup.baseUrl);
    
    // Use enhanced HTMX waiting instead of waitForSelector
    await TestHelpers.waitForHTMX(page, 10000);
    await page.waitForSelector('[data-testid="control-center"]', { timeout: 10000 });
    await page.waitForSelector('[data-testid="sidebar"]', { timeout: 10000 });
  });

  test('All buttons use same hover transform (translateY)', async ({ page }: { page: Page }) => {
    // Get unified buttons from both control center and sidebar (exclude Add Project button)
    const controlButtons = page.locator('[data-testid="control-center"] button:not(#add-project-btn)');
    const sidebarButtons = page.locator('[data-testid="project-button"]');
    
    // Wait for sidebar buttons to be visible before testing
    await expect(sidebarButtons.first()).toBeVisible({ timeout: 10000 });
    
    // Test control center buttons
    const controlCount = await controlButtons.count();
    for (let i = 0; i < controlCount; i++) {
      const button = controlButtons.nth(i);
      
      // Test hover
      await button.hover();
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
      
      // Test hover out  
      await page.mouse.move(0, 0);
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    }
    
    // Test sidebar project buttons (limit to 3 for simplicity)
    const sidebarCount = Math.min(await sidebarButtons.count(), 3);
    for (let i = 0; i < sidebarCount; i++) {
      const button = sidebarButtons.nth(i);
      
      // Test hover
      await button.hover();
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
      
      // Test hover out
      await page.mouse.move(0, 0);
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    }
  });

  test('All buttons use consistent hover colors', async ({ page }: { page: Page }) => {
    // Get sample buttons from both areas
    const buildButton = page.locator('button:has-text("Build")');
    const firstProjectButton = page.locator('[data-testid="project-button"]').first();
    
    // Ensure both buttons are visible before testing
    await expect(buildButton).toBeVisible();
    await expect(firstProjectButton).toBeVisible();
    
    // Test that both areas use hover colors (we don't hardcode specific colors, 
    // just verify they change consistently)
    
    // Get initial colors
    const buildInitialColor = await buildButton.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
    const projectInitialColor = await firstProjectButton.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
    
    // Test hover on build button
    await buildButton.hover();
    const buildHoverColor = await buildButton.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
    
    // Reset hover
    await page.mouse.move(0, 0);
    
    // Test hover on project button
    await firstProjectButton.hover();
    const projectHoverColor = await firstProjectButton.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
    
    // Reset hover
    await page.mouse.move(0, 0);
    
    // Colors should change on hover (not be the same as initial)
    expect(buildHoverColor).not.toBe(buildInitialColor);
    expect(projectHoverColor).not.toBe(projectInitialColor);
  });

  test('Consistent styling across control center and sidebar components', async ({ page }: { page: Page }) => {
    // Test that buttons in both areas respond to hover with the same transform behavior
    const controlButton = page.locator('button:has-text("Build")');
    const sidebarButton = page.locator('[data-testid="project-button"]').first();
    
    // Both should be visible
    await expect(controlButton).toBeVisible();
    await expect(sidebarButton).toBeVisible();
    
    // Both should have the same hover transform behavior
    await controlButton.hover();
    const controlTransform = await controlButton.evaluate((el: HTMLElement) => getComputedStyle(el).transform);
    await page.mouse.move(0, 0);
    
    await sidebarButton.hover();
    const sidebarTransform = await sidebarButton.evaluate((el: HTMLElement) => getComputedStyle(el).transform);
    await page.mouse.move(0, 0);
    
    // Both should have the same translateY(-1px) transform
    expect(controlTransform).toBe(sidebarTransform);
  });

  test('No buttons left in hover state after interactions', async ({ page }: { page: Page }) => {
    // Test that buttons return to normal state after hover
    const allButtons = page.locator('button, [data-testid="project-button"]');
    const buttonCount = Math.min(await allButtons.count(), 5); // Test first 5 buttons
    
    // Hover and unhover all buttons, ensuring each is visible first
    for (let i = 0; i < buttonCount; i++) {
      const button = allButtons.nth(i);
      await expect(button).toBeVisible();
      await button.hover();
      await page.mouse.move(0, 0);
    }
    
    // After all interactions, all buttons should be back to translateY(0px)
    for (let i = 0; i < buttonCount; i++) {
      const button = allButtons.nth(i);
      await expect(button).toBeVisible();
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    }
  });
});