/**
 * BaseButton Component Specification Tests
 * 
 * These tests define the contract for the BaseButton component that will be created
 * in Phase 2 of the button migration. All tests are expected to FAIL initially
 * (Red phase of TDD) until the BaseButton component is implemented.
 */

const { test, expect } = require('@playwright/test');
const { 
  renderBaseButton, 
  measureHoverResponseTime,
  getUIColor,
  hasVerticalTranslation
} = require('../helpers/ComponentTestHelpers');

test.describe('BaseButton Component Specifications', () => {
  
  test.describe('HTML Structure and Rendering', () => {
    
    test('should render button element with correct type attribute', async ({ page }) => {
      // Test configuration variants
      const configs = [
        { type: 'submit', expectedType: 'submit' },
        { type: 'button', expectedType: 'button' },
        { type: undefined, expectedType: 'button' } // default
      ];
      
      for (const config of configs) {
        // Clear previous buttons
        await page.evaluate(() => document.body.innerHTML = '');
        
        const buttonElement = await renderBaseButton(page, config);
        const actualType = await buttonElement.getAttribute('type');
        expect(actualType).toBe(config.expectedType);
      }
    });

    test('should render button content correctly', async ({ page }) => {
      const testContent = 'Test Button Content';
      const buttonElement = await renderBaseButton(page, { content: testContent });
      const actualContent = await buttonElement.textContent();
      expect(actualContent.trim()).toBe(testContent);
    });

    test('should apply CSS classes correctly', async ({ page }) => {
      const config = {
        extraClasses: 'custom-class another-class'
      };
      const buttonElement = await renderBaseButton(page, config);
      const classList = await buttonElement.getAttribute('class');
      
      expect(classList).toContain('custom-class');
      expect(classList).toContain('another-class');
      expect(classList).toContain('ftl_control'); // base class
    });

    test('should apply disabled state correctly', async ({ page }) => {
      const config = { disabled: true };
      const buttonElement = await renderBaseButton(page, config);
      const isDisabled = await buttonElement.isDisabled();
      expect(isDisabled).toBe(true);
    });

    test('should have data-test-component attribute for identification', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, {});
      const testComponent = await buttonElement.getAttribute('data-test-component');
      expect(testComponent).toBe('BaseButton');
    });
    
  });

  test.describe('HTMX Attribute Configuration', () => {
    
    test('should apply all HTMX attributes when configuration provided', async ({ page }) => {
      const htmxConfig = {
        post: '/test/endpoint',
        target: '#test-target',
        swap: 'innerHTML',
        trigger: 'click',
        vals: '{"test": "value"}',
        onRequest: 'testFunction()'
      };
      
      const config = { htmx: htmxConfig };
      const buttonElement = await renderBaseButton(page, config);
      
      expect(await buttonElement.getAttribute('hx-post')).toBe(htmxConfig.post);
      expect(await buttonElement.getAttribute('hx-target')).toBe(htmxConfig.target);
      expect(await buttonElement.getAttribute('hx-swap')).toBe(htmxConfig.swap);
      expect(await buttonElement.getAttribute('hx-trigger')).toBe(htmxConfig.trigger);
      expect(await buttonElement.getAttribute('hx-vals')).toBe(htmxConfig.vals);
      expect(await buttonElement.getAttribute('hx-on::before-request')).toBe(htmxConfig.onRequest);
    });

    test('should not apply HTMX attributes when configuration is null', async ({ page }) => {
      const config = { htmx: null };
      const buttonElement = await renderBaseButton(page, config);
      
      const htmxAttributes = [
        'hx-post', 'hx-target', 'hx-swap', 'hx-trigger', 'hx-vals', 'hx-on::before-request'
      ];
      
      for (const attr of htmxAttributes) {
        const value = await buttonElement.getAttribute(attr);
        expect(value).toBeNull();
      }
    });

    test('should apply partial HTMX configuration correctly', async ({ page }) => {
      const htmxConfig = {
        post: '/test/endpoint',
        target: '#test-target'
        // Other attributes intentionally omitted
      };
      
      const config = { htmx: htmxConfig };
      const buttonElement = await renderBaseButton(page, config);
      
      expect(await buttonElement.getAttribute('hx-post')).toBe(htmxConfig.post);
      expect(await buttonElement.getAttribute('hx-target')).toBe(htmxConfig.target);
      expect(await buttonElement.getAttribute('hx-swap')).toBeNull();
      expect(await buttonElement.getAttribute('hx-trigger')).toBeNull();
      expect(await buttonElement.getAttribute('hx-vals')).toBeNull();
      expect(await buttonElement.getAttribute('hx-on::before-request')).toBeNull();
    });

    test('should handle HTMX configuration with empty values', async ({ page }) => {
      const htmxConfig = {
        post: '',
        target: '',
        swap: null,
        trigger: undefined
      };
      
      const config = { htmx: htmxConfig };
      const buttonElement = await renderBaseButton(page, config);
      
      // Empty strings should not create attributes
      expect(await buttonElement.getAttribute('hx-post')).toBeNull();
      expect(await buttonElement.getAttribute('hx-target')).toBeNull();
      expect(await buttonElement.getAttribute('hx-swap')).toBeNull();
      expect(await buttonElement.getAttribute('hx-trigger')).toBeNull();
    });
    
  });

  test.describe('Unified Hover Behavior', () => {
    
    test('should execute hover effect on mouseover with correct CSS properties', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, {});
      
      // Get initial state
      const initialBgColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      const initialBorderColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).borderColor
      );
      const initialTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      
      // Trigger hover
      await buttonElement.hover();
      
      // Allow for transition
      await page.waitForTimeout(100);
      
      // Validate hover state changes
      const hoverBgColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      const hoverBorderColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).borderColor
      );
      const hoverTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      
      // Validate changes occurred
      expect(hoverBgColor).not.toBe(initialBgColor);
      expect(hoverBorderColor).not.toBe(initialBorderColor);
      expect(hasVerticalTranslation(hoverTransform, -1)).toBe(true);
    });

    test('should return to normal state on mouseout', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, {});
      
      // Get initial state
      const initialBgColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      const initialBorderColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).borderColor
      );
      const initialTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      
      // Trigger hover then unhover
      await buttonElement.hover();
      await page.waitForTimeout(250); // Wait for transition
      await buttonElement.dispatchEvent('mouseout'); // Trigger mouseout event directly
      await page.waitForTimeout(250); // Wait for transition back
      
      // Validate return to normal state
      const finalBgColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      const finalBorderColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).borderColor
      );
      const finalTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      
      expect(finalBgColor).toBe(initialBgColor);
      expect(finalBorderColor).toBe(initialBorderColor);
      expect(hasVerticalTranslation(finalTransform, 0)).toBe(true);
    });



    test('should not apply hover effects when disabled', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, { disabled: true });
      
      // Get initial state
      const initialBgColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      const initialTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      
      // Try to trigger hover (should be ignored for disabled buttons)
      await buttonElement.hover();
      await page.waitForTimeout(100);
      
      // Validate no changes occurred
      const hoverBgColor = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      const hoverTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      
      expect(hoverBgColor).toBe(initialBgColor);
      expect(hoverTransform).toBe(initialTransform);
    });
    
  });

  test.describe('Form Integration', () => {
    
    test('should trigger form submission when type is submit', async ({ page }) => {
      await page.setContent(`
        <form id="test-form">
          <div id="base-button-container"></div>
        </form>
      `);
      
      // Set up form submission handler
      await page.evaluate(() => {
        document.getElementById('test-form').addEventListener('submit', (e) => {
          e.preventDefault();
          window.formSubmitted = true;
        });
      });
      
      const buttonElement = await renderBaseButton(page, {
        type: 'submit',
        content: 'Submit'
      }, '#base-button-container');
      
      await buttonElement.click();
      
      const wasSubmitted = await page.evaluate(() => window.formSubmitted);
      expect(wasSubmitted).toBe(true);
    });

    test('should not trigger form submission when type is button', async ({ page }) => {
      await page.setContent(`
        <form id="test-form">
          <div id="base-button-container"></div>
        </form>
      `);
      
      // Set up form submission handler
      await page.evaluate(() => {
        document.getElementById('test-form').addEventListener('submit', (e) => {
          e.preventDefault();
          window.formSubmitted = true;
        });
      });
      
      const buttonElement = await renderBaseButton(page, {
        type: 'button',
        content: 'Button'
      }, '#base-button-container');
      
      await buttonElement.click();
      
      const wasSubmitted = await page.evaluate(() => window.formSubmitted || false);
      expect(wasSubmitted).toBe(false);
    });

    
  });

  test.describe('Accessibility and Usability', () => {
    
    test('should have proper focus behavior', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, { content: 'Focus Test' });
      
      await buttonElement.focus();
      
      const isFocused = await buttonElement.evaluate(el => 
        document.activeElement === el
      );
      expect(isFocused).toBe(true);
    });

    test('should be keyboard accessible', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, { content: 'Keyboard Test' });
      
      await buttonElement.focus();
      
      // Set up click handler to verify keyboard activation
      await page.evaluate(() => {
        window.buttonClicked = false;
        document.querySelector('button').addEventListener('click', () => {
          window.buttonClicked = true;
        });
      });
      
      // Trigger keyboard activation
      await page.keyboard.press('Enter');
      
      const wasClicked = await page.evaluate(() => window.buttonClicked);
      expect(wasClicked).toBe(true);
    });

    test('should handle rapid interactions gracefully', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, { content: 'Rapid Test' });
      
      // Rapidly hover/unhover
      for (let i = 0; i < 5; i++) {
        await buttonElement.hover();
        await page.waitForTimeout(10);
        await page.mouse.move(0, 0);
        await page.waitForTimeout(10);
      }
      
      // Should end in normal state
      await page.waitForTimeout(100);
      const finalTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      expect(hasVerticalTranslation(finalTransform, 0)).toBe(true);
    });

    test('should maintain visual consistency across different content lengths', async ({ page }) => {
      const contents = [
        'Short',
        'Medium Length Button',
        'Very Long Button Content That Should Still Look Consistent'
      ];
      
      const buttons = [];
      for (const content of contents) {
        const button = await renderBaseButton(page, { content });
        buttons.push(button);
      }
      
      // All buttons should have consistent padding and styling
      for (const button of buttons) {
        const padding = await button.evaluate(el => 
          window.getComputedStyle(el).padding
        );
        const fontSize = await button.evaluate(el => 
          window.getComputedStyle(el).fontSize
        );
        
        expect(padding).toBe('8px 16px');
        expect(fontSize).toBe('14px');
      }
    });
    
  });

  test.describe('Error Handling and Edge Cases', () => {
    
    test('should handle null/undefined configuration gracefully', async ({ page }) => {
      // Should not throw errors with minimal configuration
      const buttonElement = await renderBaseButton(page, {});
      expect(await buttonElement.isVisible()).toBe(true);
      
      const buttonElement2 = await renderBaseButton(page, null);
      expect(await buttonElement2.isVisible()).toBe(true);
    });

    test('should handle invalid HTMX configuration gracefully', async ({ page }) => {
      const invalidConfig = {
        htmx: {
          post: 123, // Invalid type
          target: [], // Invalid type
          swap: {},   // Invalid type
        }
      };
      
      // Should not throw errors, should convert to strings or ignore
      const buttonElement = await renderBaseButton(page, invalidConfig);
      expect(await buttonElement.isVisible()).toBe(true);
    });

    test('should handle empty content gracefully', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, { content: '' });
      expect(await buttonElement.isVisible()).toBe(true);
      
      const buttonElement2 = await renderBaseButton(page, { content: null });
      expect(await buttonElement2.isVisible()).toBe(true);
    });

    test('should maintain hover state consistency during DOM updates', async ({ page }) => {
      const buttonElement = await renderBaseButton(page, { content: 'DOM Test' });
      
      // Start hover
      await buttonElement.hover();
      await page.waitForTimeout(100);
      
      // Modify DOM (simulate what HTMX might do)
      await page.evaluate(() => {
        const newDiv = document.createElement('div');
        newDiv.textContent = 'New content';
        document.body.appendChild(newDiv);
      });
      
      // Hover state should be maintained
      const hoverTransform = await buttonElement.evaluate(el => 
        window.getComputedStyle(el).transform
      );
      expect(hasVerticalTranslation(hoverTransform, -1)).toBe(true);
    });
    
  });

});