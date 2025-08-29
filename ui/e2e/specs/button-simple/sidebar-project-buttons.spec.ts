import { test, expect, Page, Locator } from '@playwright/test';
import TestHelpers from '../../utils/test-helpers';

test.describe('Sidebar Project Buttons', () => {
  test.beforeEach(async ({ page }: { page: Page }) => {
    // Initialize test data with multiple projects for sidebar testing
    await TestDataManager.initialize('multiple', { count: 3 });
    await page.goto('/');
    await page.waitForSelector('[data-testid="sidebar"]', { timeout: 10000 });
  });

  test('Project buttons have consistent hover behavior', async ({ page }: { page: Page }) => {
    const projectButtons = page.locator('[data-testid="project-button"]');
    const firstButton = projectButtons.first();
    
    // Check button is visible
    await expect(firstButton).toBeVisible();
    
    // Test hover transform
    await firstButton.hover();
    await expect(firstButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
    
    // Test hover out
    await page.mouse.move(0, 0);
    await expect(firstButton).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
  });

  test('Active project has different background color', async ({ page }: { page: Page }) => {
    const activeProject = page.locator('[data-testid="project-button"][data-active="true"]');
    const inactiveProject = page.locator('[data-testid="project-button"][data-active="false"]').first();
    
    // Ensure both types exist
    await expect(activeProject).toBeVisible();
    await expect(inactiveProject).toBeVisible();
    
    // Get background colors
    const activeColor = await activeProject.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
    const inactiveColor = await inactiveProject.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
    
    // They should be different
    expect(activeColor).not.toBe(inactiveColor);
  });

  test('Status dots change color when FTL/Watch running vs stopped', async ({ page }: { page: Page }) => {
    // Test running status dot (green for regular, blue for watch)
    const runningDot = page.locator('[data-testid="status-dot"][data-status="running"]').first();
    if (await runningDot.count() > 0) {
      const runningColor = await runningDot.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
      // Should be green (regular) or blue (watch), not gray
      expect(runningColor).not.toContain('rgb(128, 128, 128)'); // not gray
    }
    
    // Test stopped status dot (gray)
    const stoppedDot = page.locator('[data-testid="status-dot"][data-status="stopped"]').first();
    if (await stoppedDot.count() > 0) {
      const stoppedColor = await stoppedDot.evaluate((el: HTMLElement) => getComputedStyle(el).backgroundColor);
      // Should be gray or similar neutral color
      expect(stoppedColor).toMatch(/rgb\((128|156|169)/); // gray variations
    }
  });

  test('Project switching changes active project correctly', async ({ page }: { page: Page }) => {
    const inactiveProject = page.locator('[data-testid="project-button"][data-active="false"]').first();
    
    // Skip if no inactive projects to click
    if (await inactiveProject.count() === 0) {
      test.skip('No inactive projects available for switching test');
    }
    
    // Get the project name before clicking
    const projectName = await inactiveProject.getAttribute('data-project-name');
    
    // Click to switch projects
    await inactiveProject.click();
    
    // Wait for the switch to complete
    await page.waitForSelector(`[data-testid="project-button"][data-project-name="${projectName}"][data-active="true"]`);
    
    // Verify the project is now active
    const nowActiveProject = page.locator(`[data-testid="project-button"][data-project-name="${projectName}"]`);
    await expect(nowActiveProject).toHaveAttribute('data-active', 'true');
  });

  test('All project buttons use consistent hover transform', async ({ page }: { page: Page }) => {
    const projectButtons = page.locator('[data-testid="project-button"]');
    const buttonCount = await projectButtons.count();
    
    // Test hover behavior on all visible project buttons (max 3 to keep it simple)
    const testCount = Math.min(buttonCount, 3);
    
    for (let i = 0; i < testCount; i++) {
      const button = projectButtons.nth(i);
      
      // Test hover
      await button.hover();
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, -1)');
      
      // Test hover out
      await page.mouse.move(0, 0);
      await expect(button).toHaveCSS('transform', 'matrix(1, 0, 0, 1, 0, 0)');
    }
  });
});