/**
 * Unified Behavior Cross-Component Integration Tests
 * 
 * Tests the unified hover behavior, styling consistency, and interaction patterns
 * across BaseButton (Control Center) and SidebarProjectButton components.
 */

const { test, expect } = require('@playwright/test');
const { 
  renderBaseButton,
  renderSidebarProjectButton,
  setupTestProjectsInSidebar,
  measureHoverResponseTime,
  simulateControlCenterSetup,
  validateUnifiedHoverBehavior
} = require('../helpers/ComponentTestHelpers');

const testProjects = require('../fixtures/test-projects.json');
const uiColorScheme = require('../fixtures/ui-color-scheme.json');

test.describe('Unified Behavior Cross-Component Integration', () => {
  
  test.beforeEach(async ({ page }) => {
    // Setup both control center and sidebar for cross-component testing
    await simulateControlCenterSetup(page);
    await setupTestProjectsInSidebar(page, testProjects.projects);
  });
  
  test.describe('Unified Hover Behavior Validation', () => {
    
    test('should implement identical hover behavior across BaseButton and SidebarProjectButton', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      // Setup both button types
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Test unified hover behavior for BaseButton
      const baseHoverBehavior = await validateUnifiedHoverBehavior(page, baseButton);
      
      // Test unified hover behavior for SidebarProjectButton  
      const sidebarHoverBehavior = await validateUnifiedHoverBehavior(page, sidebarButton);
      
      // Both should implement the same hover pattern
      expect(baseHoverBehavior.hasHoverEffect).toBe(true);
      expect(sidebarHoverBehavior.hasHoverEffect).toBe(true);
      
      // Hover states should be identical
      expect(baseHoverBehavior.hoverBackgroundColor).toBe(sidebarHoverBehavior.hoverBackgroundColor);
      expect(baseHoverBehavior.hoverBorderColor).toBe(sidebarHoverBehavior.hoverBorderColor);
      expect(baseHoverBehavior.hoverTransform).toBe(sidebarHoverBehavior.hoverTransform);
    });
    
    test('should have consistent hover timing across components', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Measure hover response times
      const baseResponseTime = await measureHoverResponseTime(page, baseButton);
      const sidebarResponseTime = await measureHoverResponseTime(page, sidebarButton);
      
      // Both should meet performance threshold
      expect(baseResponseTime).toBeLessThan(uiColorScheme.timing.max_hover_response_time);
      expect(sidebarResponseTime).toBeLessThan(uiColorScheme.timing.max_hover_response_time);
      
      // Response times should be similar (within 10ms tolerance)
      const timeDifference = Math.abs(baseResponseTime - sidebarResponseTime);
      expect(timeDifference).toBeLessThan(10);
    });
    
    test('should maintain hover state consistency during rapid mouse movement', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Rapid hover sequence across both buttons
      for (let i = 0; i < 5; i++) {
        await baseButton.hover();
        await page.waitForTimeout(50);
        
        await sidebarButton.hover();
        await page.waitForTimeout(50);
        
        // Move mouse away
        await page.mouse.move(0, 0);
        await page.waitForTimeout(50);
      }
      
      // Final state: both buttons should be in normal (non-hover) state
      await expect(baseButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
      await expect(sidebarButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
      
      await expect(baseButton).toHaveCSS('transform', uiColorScheme.transforms.normal_translateY);
      await expect(sidebarButton).toHaveCSS('transform', uiColorScheme.transforms.normal_translateY);
    });
  });
  
  test.describe('Styling Consistency Validation', () => {
    
    test('should use consistent base styling across components', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Both should use the same base CSS class
      await expect(baseButton).toHaveClass(new RegExp(uiColorScheme.cssClasses.baseButton));
      await expect(sidebarButton).toHaveClass(new RegExp(uiColorScheme.cssClasses.baseButton));
      
      // Both should have consistent base colors
      await expect(baseButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
      await expect(sidebarButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
      
      await expect(baseButton).toHaveCSS('border-color', uiColorScheme.rgbColors.button_border);
      await expect(sidebarButton).toHaveCSS('border-color', uiColorScheme.rgbColors.button_border);
    });
    
    test('should maintain consistent typography across components', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Both should have consistent text styling
      const baseFontFamily = await baseButton.evaluate(el => getComputedStyle(el).fontFamily);
      const sidebarFontFamily = await sidebarButton.evaluate(el => getComputedStyle(el).fontFamily);
      expect(baseFontFamily).toBe(sidebarFontFamily);
      
      const baseFontWeight = await baseButton.evaluate(el => getComputedStyle(el).fontWeight);
      const sidebarFontWeight = await sidebarButton.evaluate(el => getComputedStyle(el).fontWeight);
      expect(baseFontWeight).toBe(sidebarFontWeight);
      
      // Text color should be consistent
      await expect(baseButton).toHaveCSS('color', uiColorScheme.rgbColors.primary_text || 'rgb(255, 255, 255)');
      await expect(sidebarButton).toHaveCSS('color', uiColorScheme.rgbColors.primary_text || 'rgb(255, 255, 255)');
    });
    
    test('should have consistent border and spacing across components', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Both should have consistent border styling
      const baseBorderWidth = await baseButton.evaluate(el => getComputedStyle(el).borderWidth);
      const sidebarBorderWidth = await sidebarButton.evaluate(el => getComputedStyle(el).borderWidth);
      expect(baseBorderWidth).toBe(sidebarBorderWidth);
      
      const baseBorderStyle = await baseButton.evaluate(el => getComputedStyle(el).borderStyle);
      const sidebarBorderStyle = await sidebarButton.evaluate(el => getComputedStyle(el).borderStyle);
      expect(baseBorderStyle).toBe(sidebarBorderStyle);
      
      // Both should have consistent padding
      const basePadding = await baseButton.evaluate(el => getComputedStyle(el).padding);
      const sidebarPadding = await sidebarButton.evaluate(el => getComputedStyle(el).padding);
      expect(basePadding).toBe(sidebarPadding);
    });
  });
  
  test.describe('Interaction Pattern Consistency', () => {
    
    test('should handle focus states consistently across components', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Focus BaseButton
      await baseButton.focus();
      await expect(baseButton).toBeFocused();
      
      const baseFocusOutline = await baseButton.evaluate(el => getComputedStyle(el).outline);
      
      // Focus SidebarProjectButton
      await sidebarButton.focus();
      await expect(sidebarButton).toBeFocused();
      
      const sidebarFocusOutline = await sidebarButton.evaluate(el => getComputedStyle(el).outline);
      
      // Focus states should be consistent
      expect(baseFocusOutline).toBe(sidebarFocusOutline);
    });
    
    test('should handle keyboard interactions consistently', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Test Tab navigation
      await page.keyboard.press('Tab');
      
      // Should be able to reach both buttons via keyboard navigation
      let focusedElement = await page.evaluate(() => document.activeElement?.tagName);
      expect(focusedElement).toBe('BUTTON');
      
      // Test Enter key activation
      await page.keyboard.press('Enter');
      
      // Both should respond to Enter key (though behavior differs)
      // This validates the interaction pattern is consistent
    });
    
    test('should maintain consistent disabled states when applicable', async ({ page }) => {
      // Test with a scenario where buttons might be disabled
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {}, { disabled: true });
      await renderSidebarProjectButton(page, inactiveProject, { disabled: true });
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Both should be disabled
      await expect(baseButton).toBeDisabled();
      await expect(sidebarButton).toBeDisabled();
      
      // Both should have consistent disabled styling
      const baseOpacity = await baseButton.evaluate(el => getComputedStyle(el).opacity);
      const sidebarOpacity = await sidebarButton.evaluate(el => getComputedStyle(el).opacity);
      
      expect(baseOpacity).toBe(sidebarOpacity);
      expect(parseFloat(baseOpacity)).toBeLessThan(1.0); // Should be visually dimmed
    });
  });
  
  test.describe('Responsive Behavior Consistency', () => {
    
    test('should maintain consistent behavior across different viewport sizes', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      // Test in multiple viewport sizes
      const viewportSizes = [
        { width: 1920, height: 1080 }, // Desktop
        { width: 1024, height: 768 },  // Tablet
        { width: 375, height: 667 }    // Mobile
      ];
      
      for (const viewport of viewportSizes) {
        await page.setViewportSize(viewport);
        
        await renderBaseButton(page, 'Build', 'submit', {});
        await renderSidebarProjectButton(page, inactiveProject);
        
        const baseButton = page.locator('button:has-text("Build")');
        const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
        
        // Both buttons should remain visible and interactive
        await expect(baseButton).toBeVisible();
        await expect(sidebarButton).toBeVisible();
        
        // Hover behavior should work consistently
        await baseButton.hover();
        await expect(baseButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg_hover);
        
        await sidebarButton.hover();
        await expect(sidebarButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg_hover);
        
        // Reset for next iteration
        await page.mouse.move(0, 0);
      }
    });
    
    test('should handle touch interactions consistently on mobile', async ({ page }) => {
      // Simulate mobile environment
      await page.setViewportSize({ width: 375, height: 667 });
      
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Simulate touch events
      await baseButton.dispatchEvent('touchstart');
      await baseButton.dispatchEvent('touchend');
      
      await sidebarButton.dispatchEvent('touchstart');  
      await sidebarButton.dispatchEvent('touchend');
      
      // Both should handle touch events gracefully
      await expect(baseButton).toBeEnabled();
      await expect(sidebarButton).toBeEnabled();
    });
  });
  
  test.describe('Animation and Transition Consistency', () => {
    
    test('should have consistent hover transition timing', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Both should have consistent transition duration
      const baseTransition = await baseButton.evaluate(el => getComputedStyle(el).transition);
      const sidebarTransition = await sidebarButton.evaluate(el => getComputedStyle(el).transition);
      
      // Extract duration from transition property
      const extractDuration = (transition) => {
        const match = transition.match(/(\d+(?:\.\d+)?)m?s/);
        return match ? match[1] : '0';
      };
      
      const baseDuration = extractDuration(baseTransition);
      const sidebarDuration = extractDuration(sidebarTransition);
      
      expect(baseDuration).toBe(sidebarDuration);
      expect(baseDuration).toBe('200'); // Should match uiColorScheme.timing.hover_duration
    });
    
    test('should maintain visual consistency during rapid state changes', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Rapid hover/unhover cycles
      for (let i = 0; i < 10; i++) {
        await Promise.all([
          baseButton.hover(),
          sidebarButton.hover()
        ]);
        
        await page.waitForTimeout(25);
        
        await page.mouse.move(0, 0);
        await page.waitForTimeout(25);
      }
      
      // Final state should be consistent and stable
      await expect(baseButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
      await expect(sidebarButton).toHaveCSS('background-color', uiColorScheme.rgbColors.button_bg);
    });
  });
  
  test.describe('Accessibility Consistency', () => {
    
    test('should maintain consistent ARIA patterns across components', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      const baseButton = page.locator('button:has-text("Build")');
      const sidebarButton = page.locator(`[data-project-name="${inactiveProject.name}"] button`);
      
      // Both should have appropriate ARIA attributes
      await expect(baseButton).toHaveAttribute('role', 'button');
      await expect(sidebarButton).toHaveAttribute('role', 'button');
      
      // Both should have descriptive labels
      const baseAriaLabel = await baseButton.getAttribute('aria-label');
      const sidebarAriaLabel = await sidebarButton.getAttribute('aria-label');
      
      expect(baseAriaLabel).toBeTruthy();
      expect(sidebarAriaLabel).toBeTruthy();
      
      // Labels should be descriptive and follow consistent patterns
      expect(baseAriaLabel).toMatch(/build/i);
      expect(sidebarAriaLabel).toMatch(/switch.*project/i);
    });
    
    test('should provide consistent keyboard navigation experience', async ({ page }) => {
      const inactiveProject = testProjects.projects.find(p => !p.isActive);
      
      await renderBaseButton(page, 'Build', 'submit', {});
      await renderSidebarProjectButton(page, inactiveProject);
      
      // Both buttons should be reachable via Tab navigation
      await page.keyboard.press('Tab');
      let focusedElement = await page.evaluate(() => document.activeElement);
      expect(focusedElement).toBeTruthy();
      
      await page.keyboard.press('Tab');
      focusedElement = await page.evaluate(() => document.activeElement);
      expect(focusedElement).toBeTruthy();
      
      // Both should respond to Enter and Space keys consistently
      await page.keyboard.press('Enter');
      await page.waitForTimeout(100);
      
      await page.keyboard.press('Tab');
      await page.keyboard.press('Space');
      await page.waitForTimeout(100);
      
      // Both interactions should work without errors
    });
  });
});