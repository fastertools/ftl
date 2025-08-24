// @ts-check
const { defineConfig, devices } = require('@playwright/test');
const fs = require('fs');
const path = require('path');

// Try to load dynamic test config if available
let baseURL = 'http://localhost:8080';
const testConfigPath = path.join(__dirname, 'e2e-tests', '.test-config.json');
if (fs.existsSync(testConfigPath)) {
  const config = JSON.parse(fs.readFileSync(testConfigPath, 'utf8'));
  baseURL = config.baseURL || baseURL;
  console.log(`Using dynamic baseURL: ${baseURL}`);
}

/**
 * @see https://playwright.dev/docs/test-configuration
 */
module.exports = defineConfig({
  testDir: './e2e-tests/e2e/specs',
  timeout: 30 * 1000,
  expect: {
    timeout: 5000
  },
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list']
  ],
  globalSetup: './e2e-tests/global-setup.js',
  globalTeardown: './e2e-tests/global-teardown.js',
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  outputDir: 'test-results/',
});