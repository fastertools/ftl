const { test, expect } = require('@playwright/test');
const DashboardPage = require('../pages/DashboardPage');
const TestFixtures = require('../fixtures/TestFixtures');
const TestHelpers = require('../utils/TestHelpers');

test.describe('Dashboard Tests', () => {
    let fixtures;
    let page;
    let dashboardPage;

    test.beforeAll(async () => {
        fixtures = new TestFixtures();
        page = await fixtures.setup({ headless: true });
        dashboardPage = new DashboardPage(page);
    });

    test.afterAll(async () => {
        await fixtures.teardown();
    });

    test.beforeEach(async () => {
        await dashboardPage.navigate();
    });

    test('should load dashboard successfully', async () => {
        const title = await page.title();
        expect(title).toContain('FTL');
    });

    test('should have Build button', async () => {
        const buildButton = await page.locator(dashboardPage.buildButton).first();
        expect(await buildButton.count()).toBeGreaterThan(0);
    });

    test.skip('should execute Build command', async () => {
        // Skipped: Build won't work in a random folder
        const response = await dashboardPage.clickBuild();
        expect(response.status()).toBe(200);
        
        // Wait for output to appear
        await dashboardPage.waitForPolling(2000);
        const output = await dashboardPage.getCommandOutput();
        expect(output).toBeTruthy();
    });

    test('should have FTL Up button', async () => {
        const ftlButton = await page.locator(dashboardPage.ftlUpButton).first();
        expect(await ftlButton.count()).toBeGreaterThan(0);
    });

    test('should not navigate on button clicks (HTMX)', async () => {
        const navigationOccurred = await TestHelpers.checkForNavigation(page, async () => {
            await dashboardPage.clickBuild();
        });
        expect(navigationOccurred).toBe(false);
    });

    test.skip('should show process status', async () => {
        // Skipped: Requires actual FTL processes running
        await dashboardPage.waitForPolling(3000);
        const status = await dashboardPage.getProcessStatus();
        expect(status).toBeTruthy();
    });
});