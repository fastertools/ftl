// HTMXHelpers.ts - Utilities for HTMX synchronization in tests
// Replaces arbitrary timeouts with deterministic waits using Go test endpoints

import { Page, APIRequestContext } from '@playwright/test';

interface HTMXOptions {
  timeout?: number;
  projectPath?: string;
}

interface StatusResult {
  ready: boolean;
  status?: string;
  message?: string;
}

interface LogsResult {
  ready: boolean;
  logs: string[];
  message?: string;
}

interface ProcessTreeResult {
  ftl: { is_running: boolean };
  watch: { is_running: boolean };
}

class HTMXHelpers {
  /**
   * Wait for HTMX operations to settle (no active polling)
   * Replaces arbitrary timeouts with deterministic waiting
   */
  static async waitForSettle(page: Page, options: HTMXOptions = {}): Promise<boolean> {
    const timeout = options.timeout || 5000;
    const projectPath = options.projectPath || (page.context() as any).projectPath || '/tmp/test-project';
    
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
   */
  static async waitForStatus(page: Page, expectedStatus: string, options: HTMXOptions = {}): Promise<StatusResult> {
    const timeout = options.timeout || 10000;
    const projectPath = options.projectPath || (page.context() as any).projectPath || '/tmp/test-project';
    
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
        message: (error as Error).message
      };
    }
  }
  
  /**
   * Wait for logs to appear
   */
  static async waitForLogs(page: Page, minLines: number = 1, options: HTMXOptions = {}): Promise<LogsResult> {
    const timeout = options.timeout || 5000;
    const projectPath = options.projectPath || (page.context() as any).projectPath || '/tmp/test-project';
    
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
        message: (error as Error).message
      };
    }
  }
  
  /**
   * Wait for a form submission to complete with HTMX
   */
  static async waitForFormSubmission(page: Page, submitAction: () => Promise<void>, options: HTMXOptions = {}): Promise<boolean> {
    // Execute the submission action
    await submitAction();
    
    // Wait for HTMX to process the request
    return await this.waitForSettle(page, options);
  }
  
  /**
   * Wait for polling to update status
   * Combines HTMX wait with status check
   */
  static async waitForPollingUpdate(page: Page, expectedStatus: string, options: HTMXOptions = {}): Promise<boolean> {
    // First wait for any active HTMX operations to complete
    await this.waitForSettle(page, options);
    
    // Then check if status matches expected
    const statusResult = await this.waitForStatus(page, expectedStatus, options);
    
    return statusResult.ready && statusResult.status === expectedStatus;
  }
  
  /**
   * Get current process tree information
   */
  static async getProcessTree(page: Page, projectPath?: string): Promise<ProcessTreeResult> {
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
   */
  static async waitForElement(page: Page, selector: string, options: HTMXOptions = {}): Promise<boolean> {
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
   */
  static async clickAndWait(page: Page, selector: string, options: HTMXOptions = {}): Promise<boolean> {
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

export default HTMXHelpers;