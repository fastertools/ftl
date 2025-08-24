// HTMXHelpers.js - Utilities for HTMX synchronization in tests
// Replaces arbitrary timeouts with deterministic waits using Go test endpoints

class HTMXHelpers {
  /**
   * Wait for HTMX operations to settle (no active polling)
   * Replaces arbitrary timeouts with deterministic waiting
   * @param {Page} page - Playwright page object
   * @param {Object} options - Configuration options
   * @param {number} options.timeout - Maximum wait time in ms (default: 5000)
   * @param {string} options.projectPath - Project path for context
   * @returns {Promise<boolean>} - True if HTMX settled, false on timeout
   */
  static async waitForSettle(page, options = {}) {
    const timeout = options.timeout || 5000;
    const projectPath = options.projectPath || page.context().projectPath || '/tmp/test-project';
    
    try {
      // Call the Go test endpoint that checks for HTMX activity
      const response = await page.request.get('/api/test/wait-for-htmx', {
        params: {
          timeout: timeout.toString(),
          project_path: projectPath
        }
      });
      
      const result = await response.json();
      
      if (!result.ready) {
        console.warn(`HTMX wait timeout: ${result.message}`);
      }
      
      return result.ready;
    } catch (error) {
      console.error('Error waiting for HTMX to settle:', error);
      // Fallback to small delay if endpoint fails
      await page.waitForTimeout(500);
      return false;
    }
  }
  
  /**
   * Wait for process status to match expected value
   * @param {Page} page - Playwright page object
   * @param {string} expectedStatus - Expected status ('running' or 'stopped')
   * @param {Object} options - Configuration options
   * @param {number} options.timeout - Maximum wait time in ms (default: 10000)
   * @param {string} options.projectPath - Project path for context
   * @returns {Promise<Object>} - Status result object
   */
  static async waitForStatus(page, expectedStatus, options = {}) {
    const timeout = options.timeout || 10000;
    const projectPath = options.projectPath || page.context().projectPath || '/tmp/test-project';
    
    try {
      const response = await page.request.get('/api/test/wait-for-status', {
        params: {
          status: expectedStatus,
          timeout: timeout.toString(),
          project_path: projectPath
        }
      });
      
      const result = await response.json();
      
      if (!result.ready) {
        console.warn(`Status wait timeout: ${result.message}`);
      }
      
      return result;
    } catch (error) {
      console.error('Error waiting for status:', error);
      return {
        ready: false,
        status: 'unknown',
        message: error.message
      };
    }
  }
  
  /**
   * Wait for logs to appear
   * @param {Page} page - Playwright page object
   * @param {number} minLines - Minimum number of log lines to wait for
   * @param {Object} options - Configuration options
   * @param {number} options.timeout - Maximum wait time in ms (default: 5000)
   * @param {string} options.projectPath - Project path for context
   * @returns {Promise<Object>} - Logs result object with ready flag and logs array
   */
  static async waitForLogs(page, minLines = 1, options = {}) {
    const timeout = options.timeout || 5000;
    const projectPath = options.projectPath || page.context().projectPath || '/tmp/test-project';
    
    try {
      const response = await page.request.get('/api/test/wait-for-logs', {
        params: {
          min_lines: minLines.toString(),
          timeout: timeout.toString(),
          project_path: projectPath
        }
      });
      
      const result = await response.json();
      
      if (!result.ready) {
        console.warn(`Logs wait timeout: ${result.message}`);
      }
      
      return result;
    } catch (error) {
      console.error('Error waiting for logs:', error);
      return {
        ready: false,
        logs: [],
        message: error.message
      };
    }
  }
  
  /**
   * Wait for a form submission to complete with HTMX
   * @param {Page} page - Playwright page object
   * @param {function} submitAction - Function that triggers form submission
   * @param {Object} options - Configuration options
   * @returns {Promise<boolean>} - True if submission completed successfully
   */
  static async waitForFormSubmission(page, submitAction, options = {}) {
    // Execute the submission action
    await submitAction();
    
    // Wait for HTMX to process the request
    return await this.waitForSettle(page, options);
  }
  
  /**
   * Wait for polling to update status
   * Combines HTMX wait with status check
   * @param {Page} page - Playwright page object
   * @param {string} expectedStatus - Expected status after polling
   * @param {Object} options - Configuration options
   * @returns {Promise<boolean>} - True if status matches after polling
   */
  static async waitForPollingUpdate(page, expectedStatus, options = {}) {
    // First wait for any active HTMX operations to complete
    await this.waitForSettle(page, options);
    
    // Then check if status matches expected
    const statusResult = await this.waitForStatus(page, expectedStatus, options);
    
    return statusResult.ready && statusResult.status === expectedStatus;
  }
  
  /**
   * Get current process tree information
   * @param {Page} page - Playwright page object
   * @param {string} projectPath - Project path for context
   * @returns {Promise<Object>} - Process tree information
   */
  static async getProcessTree(page, projectPath) {
    try {
      const response = await page.request.get('/api/test/process-tree', {
        params: {
          project_path: projectPath || '/tmp/test-project'
        }
      });
      
      return await response.json();
    } catch (error) {
      console.error('Error getting process tree:', error);
      return {
        ftl: { is_running: false },
        watch: { is_running: false }
      };
    }
  }
  
  /**
   * Wait for a specific element to appear after HTMX update
   * @param {Page} page - Playwright page object
   * @param {string} selector - CSS selector for element
   * @param {Object} options - Configuration options
   * @returns {Promise<boolean>} - True if element appeared
   */
  static async waitForElement(page, selector, options = {}) {
    const timeout = options.timeout || 5000;
    
    try {
      // Wait for HTMX to settle first
      await this.waitForSettle(page, options);
      
      // Then wait for the element to appear
      await page.waitForSelector(selector, { 
        timeout: timeout,
        state: 'visible'
      });
      
      return true;
    } catch (error) {
      console.warn(`Element ${selector} did not appear within ${timeout}ms`);
      return false;
    }
  }
  
  /**
   * Click a button and wait for HTMX to complete processing
   * @param {Page} page - Playwright page object
   * @param {string} selector - Button selector
   * @param {Object} options - Configuration options
   * @returns {Promise<boolean>} - True if click and processing completed
   */
  static async clickAndWait(page, selector, options = {}) {
    try {
      // Click the button
      await page.click(selector);
      
      // Wait for HTMX to process
      return await this.waitForSettle(page, options);
    } catch (error) {
      console.error(`Error clicking ${selector}:`, error);
      return false;
    }
  }
}

module.exports = HTMXHelpers;