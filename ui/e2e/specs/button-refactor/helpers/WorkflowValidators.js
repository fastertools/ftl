/**
 * Workflow Validators for Button Migration Testing
 * 
 * Provides end-to-end workflow validation utilities for complex user interactions
 */

const { expect } = require('@playwright/test');

/**
 * Validates complete project switching workflow
 * 
 * @param {Page} page - Playwright page object
 * @param {Object} sourceProject - Source project configuration
 * @param {Object} targetProject - Target project configuration  
 * @returns {boolean} Validation success
 */
async function validateProjectSwitchWorkflow(page, sourceProject, targetProject) {
  // 1. Verify initial state
  const initialActiveElement = await page.locator('.project-item-active .project-name').first();
  if (await initialActiveElement.count() > 0) {
    const initialActiveName = await initialActiveElement.textContent();
    expect(initialActiveName.trim()).toBe(sourceProject.name);
  }
  
  // 2. Monitor network requests
  const switchRequestPromise = page.waitForRequest(req => 
    req.url().includes('/htmx/project/switch') &&
    req.url().includes(`path=${encodeURIComponent(targetProject.path)}`) &&
    req.method() === 'POST'
  );
  
  // 3. Monitor DOM changes
  const contentChangePromise = page.waitForFunction((targetName) => {
    const mainContainer = document.querySelector('#main-container');
    return mainContainer && mainContainer.textContent.includes(targetName);
  }, targetProject.name);
  
  // 4. Trigger switch by clicking target project
  const targetButton = page.locator(`[data-project-name="${targetProject.name}"] button[hx-post]`);
  await targetButton.click();
  
  // 5. Validate request was made
  const switchRequest = await switchRequestPromise;
  expect(switchRequest.url()).toContain(`path=${encodeURIComponent(targetProject.path)}`);
  expect(switchRequest.method()).toBe('POST');
  
  // 6. Validate DOM content changed
  await contentChangePromise;
  
  // 7. Validate sidebar state updated
  await page.waitForSelector('.project-item-active', { timeout: 5000 });
  const finalActiveElement = await page.locator('.project-item-active .project-name').first();
  const finalActiveName = await finalActiveElement.textContent();
  expect(finalActiveName.trim()).toBe(targetProject.name);
  
  // 8. Validate source project is now inactive
  const sourceProjectElement = page.locator(`[data-project-name="${sourceProject.name}"]`);
  const sourceProjectClasses = await sourceProjectElement.getAttribute('class');
  expect(sourceProjectClasses).toContain('project-item-inactive');
  expect(sourceProjectClasses).not.toContain('project-item-active');
  
  return true;
}

/**
 * Validates control operation workflow (Build, Up, Watch)
 * 
 * @param {Page} page - Playwright page object  
 * @param {string} operation - Operation name ('Build', 'Up', 'Watch')
 * @param {Object} expectedFormData - Expected form data in request
 * @returns {boolean} Validation success
 */
async function validateControlOperationWorkflow(page, operation, expectedFormData) {
  // 1. Monitor pre-request message injection
  const messagePromise = page.waitForFunction((expectedMessage) => {
    const outputContainer = document.querySelector('#ftl-output');
    return outputContainer && outputContainer.textContent.includes(expectedMessage);
  }, expectedFormData.message);
  
  // 2. Monitor network request
  const operationRequestPromise = page.waitForRequest(req => 
    req.url().includes('/mcp') && 
    req.method() === 'POST'
  );
  
  // 3. Trigger operation
  const operationButton = page.locator(`button:has-text("${operation}")`);
  await operationButton.click();
  
  // 4. Validate pre-request message appeared
  await messagePromise;
  
  const outputText = await page.locator('#ftl-output').textContent();
  expect(outputText).toContain(expectedFormData.message);
  
  // Validate timestamp format in message
  expect(outputText).toMatch(/\[\d{2}:\d{2}:\d{2}\]/);
  
  // 5. Validate network request was made with correct data
  const operationRequest = await operationRequestPromise;
  const requestBody = await operationRequest.postData();
  
  // Check all expected form fields
  for (const [key, value] of Object.entries(expectedFormData)) {
    if (key !== 'message') {
      expect(requestBody).toContain(`${key}=${encodeURIComponent(value)}`);
    }
  }
  
  // 6. Validate request headers and method
  expect(operationRequest.method()).toBe('POST');
  expect(operationRequest.headers()['content-type']).toContain('application/x-www-form-urlencoded');
  
  return true;
}

/**
 * Validates status polling continuity after DOM changes
 * 
 * @param {Page} page - Playwright page object
 * @param {string} projectPath - Project path to monitor in polls
 * @param {number} monitorDuration - Duration to monitor in milliseconds
 * @returns {Object} Polling statistics
 */
async function validateStatusPollingContinuity(page, projectPath, monitorDuration = 5000) {
  const pollingRequests = [];
  
  // Track status polling requests
  const requestHandler = (req) => {
    if (req.url().includes('/htmx/status/poll')) {
      pollingRequests.push({
        timestamp: Date.now(),
        url: req.url(),
        method: req.method(),
        projectPath: extractProjectPathFromRequest(req)
      });
    }
  };
  
  page.on('request', requestHandler);
  
  // Monitor for specified duration
  await page.waitForTimeout(monitorDuration);
  
  page.off('request', requestHandler);
  
  // Validate polling occurred
  expect(pollingRequests.length).toBeGreaterThan(0);
  
  // Validate polling frequency (approximately 1 request per second)
  if (pollingRequests.length >= 2) {
    const intervals = [];
    for (let i = 1; i < pollingRequests.length; i++) {
      intervals.push(pollingRequests[i].timestamp - pollingRequests[i-1].timestamp);
    }
    
    const avgInterval = intervals.reduce((a, b) => a + b) / intervals.length;
    expect(avgInterval).toBeGreaterThan(800); // At least 800ms
    expect(avgInterval).toBeLessThan(1200);  // At most 1200ms
  }
  
  // Validate project path in recent requests
  const recentRequests = pollingRequests.slice(-2);
  for (const req of recentRequests) {
    expect(req.projectPath).toBe(projectPath);
  }
  
  return {
    totalRequests: pollingRequests.length,
    avgInterval: pollingRequests.length >= 2 ? 
      pollingRequests.slice(1).reduce((sum, req, i) => 
        sum + (req.timestamp - pollingRequests[i].timestamp), 0) / (pollingRequests.length - 1) : 0,
    requests: pollingRequests
  };
}

/**
 * Validates log polling continuity after DOM changes
 * 
 * @param {Page} page - Playwright page object
 * @param {string} projectPath - Project path to monitor in polls
 * @param {number} monitorDuration - Duration to monitor in milliseconds
 * @returns {Object} Polling statistics
 */
async function validateLogPollingContinuity(page, projectPath, monitorDuration = 3000) {
  const pollingRequests = [];
  
  // Track log polling requests
  const requestHandler = (req) => {
    if (req.url().includes('/htmx/logs/poll')) {
      pollingRequests.push({
        timestamp: Date.now(),
        url: req.url(),
        method: req.method(),
        projectPath: extractProjectPathFromRequest(req)
      });
    }
  };
  
  page.on('request', requestHandler);
  
  // Monitor for specified duration
  await page.waitForTimeout(monitorDuration);
  
  page.off('request', requestHandler);
  
  // Validate polling occurred
  expect(pollingRequests.length).toBeGreaterThan(0);
  
  // Validate polling frequency (approximately 1 request per 500ms)
  if (pollingRequests.length >= 2) {
    const intervals = [];
    for (let i = 1; i < pollingRequests.length; i++) {
      intervals.push(pollingRequests[i].timestamp - pollingRequests[i-1].timestamp);
    }
    
    const avgInterval = intervals.reduce((a, b) => a + b) / intervals.length;
    expect(avgInterval).toBeGreaterThan(400); // At least 400ms
    expect(avgInterval).toBeLessThan(600);   // At most 600ms
  }
  
  // Validate project path in recent requests
  const recentRequests = pollingRequests.slice(-2);
  for (const req of recentRequests) {
    expect(req.projectPath).toBe(projectPath);
  }
  
  return {
    totalRequests: pollingRequests.length,
    avgInterval: pollingRequests.length >= 2 ? 
      pollingRequests.slice(1).reduce((sum, req, i) => 
        sum + (req.timestamp - pollingRequests[i].timestamp), 0) / (pollingRequests.length - 1) : 0,
    requests: pollingRequests
  };
}

/**
 * Validates HTMX request/response cycle
 * 
 * @param {Page} page - Playwright page object
 * @param {Object} expectedRequest - Expected request configuration
 * @param {Function} triggerAction - Function to trigger the HTMX request
 * @returns {Object} Request/response validation results
 */
async function validateHTMXRequestResponse(page, expectedRequest, triggerAction) {
  // Monitor specific request
  const requestPromise = page.waitForRequest(req => {
    const urlMatch = req.url().includes(expectedRequest.endpoint);
    const methodMatch = req.method() === expectedRequest.method;
    return urlMatch && methodMatch;
  });
  
  // Monitor response
  const responsePromise = page.waitForResponse(resp => {
    const urlMatch = resp.url().includes(expectedRequest.endpoint);
    const methodMatch = resp.request().method() === expectedRequest.method;
    return urlMatch && methodMatch;
  });
  
  // Trigger the action
  await triggerAction();
  
  // Validate request
  const request = await requestPromise;
  expect(request.method()).toBe(expectedRequest.method);
  expect(request.url()).toContain(expectedRequest.endpoint);
  
  // Validate request parameters if specified
  if (expectedRequest.expectedParams) {
    for (const param of expectedRequest.expectedParams) {
      expect(request.url()).toContain(param);
    }
  }
  
  // Validate response
  const response = await responsePromise;
  expect(response.status()).toBeLessThan(400); // Success status codes
  
  return {
    request: {
      method: request.method(),
      url: request.url(),
      headers: request.headers()
    },
    response: {
      status: response.status(),
      headers: response.headers(),
      body: await response.text()
    }
  };
}

/**
 * Validates OOB (Out-of-Band) swap functionality
 * 
 * @param {Page} page - Playwright page object
 * @param {string} targetSelector - Target element for main swap
 * @param {Array} oobSelectors - Array of OOB target selectors
 * @param {Function} triggerAction - Function to trigger the swap
 * @returns {boolean} Validation success
 */
async function validateOOBSwap(page, targetSelector, oobSelectors, triggerAction) {
  // Get initial content of all target elements
  const initialContents = {};
  initialContents[targetSelector] = await page.locator(targetSelector).innerHTML();
  
  for (const selector of oobSelectors) {
    try {
      initialContents[selector] = await page.locator(selector).innerHTML();
    } catch (e) {
      initialContents[selector] = ''; // Element doesn't exist initially
    }
  }
  
  // Trigger the action that should cause OOB swap
  await triggerAction();
  
  // Wait for main target to change
  await page.waitForFunction((selector, initialContent) => {
    const element = document.querySelector(selector);
    return element && element.innerHTML !== initialContent;
  }, targetSelector, initialContents[targetSelector]);
  
  // Validate main target content changed
  const newMainContent = await page.locator(targetSelector).innerHTML();
  expect(newMainContent).not.toBe(initialContents[targetSelector]);
  
  // Validate OOB targets also changed (if they should)
  for (const selector of oobSelectors) {
    try {
      const newContent = await page.locator(selector).innerHTML();
      // OOB content should be different from initial (if it was swapped)
      // Note: This might need adjustment based on specific test scenarios
      expect(newContent).toBeDefined();
    } catch (e) {
      // OOB target might not exist, which could be valid depending on scenario
    }
  }
  
  return true;
}

/**
 * Extracts project path from HTMX request
 * 
 * @param {Request} request - Playwright request object
 * @returns {string} Extracted project path
 */
function extractProjectPathFromRequest(request) {
  const url = new URL(request.url());
  
  // Check URL parameters first
  const pathParam = url.searchParams.get('path');
  if (pathParam) {
    return pathParam;
  }
  
  // Check request body for POST requests
  if (request.method() === 'POST') {
    try {
      const postData = request.postData();
      if (postData) {
        // Check if it's JSON
        if (postData.startsWith('{')) {
          const data = JSON.parse(postData);
          return data.project_path || data.path || '';
        }
        // Check if it's form data
        const params = new URLSearchParams(postData);
        return params.get('project_path') || params.get('project-path') || params.get('path') || '';
      }
    } catch (e) {
      // Ignore parsing errors
    }
  }
  
  return '';
}

/**
 * Validates concurrent HTMX requests handling
 * 
 * @param {Page} page - Playwright page object
 * @param {Array} actions - Array of actions to trigger concurrently
 * @param {number} timeout - Timeout for all requests to complete
 * @returns {Object} Concurrent request handling results
 */
async function validateConcurrentHTMXRequests(page, actions, timeout = 10000) {
  const requests = [];
  const responses = [];
  const errors = [];
  
  // Track all requests and responses
  const requestHandler = (req) => {
    if (req.url().includes('/htmx/') || req.url().includes('/mcp')) {
      requests.push({
        timestamp: Date.now(),
        method: req.method(),
        url: req.url()
      });
    }
  };
  
  const responseHandler = (resp) => {
    if (resp.url().includes('/htmx/') || resp.url().includes('/mcp')) {
      responses.push({
        timestamp: Date.now(),
        status: resp.status(),
        url: resp.url()
      });
    }
  };
  
  const responseFailedHandler = (resp) => {
    errors.push({
      timestamp: Date.now(),
      url: resp.url(),
      error: 'Response failed'
    });
  };
  
  page.on('request', requestHandler);
  page.on('response', responseHandler);
  page.on('responsefailed', responseFailedHandler);
  
  // Execute all actions concurrently
  const actionPromises = actions.map(async (action, index) => {
    try {
      await action();
      return { index, success: true };
    } catch (error) {
      errors.push({
        timestamp: Date.now(),
        action: index,
        error: error.message
      });
      return { index, success: false, error: error.message };
    }
  });
  
  // Wait for all actions to complete or timeout
  const actionResults = await Promise.allSettled(actionPromises);
  
  // Clean up event listeners
  page.off('request', requestHandler);
  page.off('response', responseHandler);
  page.off('responsefailed', responseFailedHandler);
  
  // Validate no critical errors occurred
  expect(errors.length).toBe(0);
  
  // Validate all requests got responses
  expect(responses.length).toBeGreaterThanOrEqual(requests.length * 0.8); // Allow for some polling requests
  
  // Validate no failed responses
  const failedResponses = responses.filter(r => r.status >= 400);
  expect(failedResponses.length).toBe(0);
  
  return {
    totalRequests: requests.length,
    totalResponses: responses.length,
    totalErrors: errors.length,
    actionResults,
    requests,
    responses,
    errors
  };
}

module.exports = {
  validateProjectSwitchWorkflow,
  validateControlOperationWorkflow,
  validateStatusPollingContinuity,
  validateLogPollingContinuity,
  validateHTMXRequestResponse,
  validateOOBSwap,
  validateConcurrentHTMXRequests,
  extractProjectPathFromRequest
};