// Page Object for the main dashboard
const HTMXHelpers = require('../utils/HTMXHelpers');

class DashboardPage {
    constructor(page) {
        this.page = page;
        this.url = 'http://localhost:8080';
        
        // Selectors
        this.buildButton = 'button:has-text("Build")';
        this.ftlUpButton = 'button:has-text("FTL Up"), button:has-text("Up")';
        this.watchButton = 'button:has-text("Watch")';
        this.stopButton = 'button:has-text("Stop")';
        this.logOutput = '#log-output';
        this.commandOutput = '#command-output';
        this.processStatus = '#process-status';
    }

    async navigate() {
        await this.page.goto(this.url, { waitUntil: 'domcontentloaded' });
    }

    async clickBuild() {
        const button = await this.page.locator(this.buildButton).first();
        await button.click();
        // Use HTMXHelpers for deterministic wait instead of arbitrary timeout
        await HTMXHelpers.waitForSettle(this.page, {
            projectPath: this.page.context().projectPath
        });
    }

    async clickFTLUp() {
        const button = await this.page.locator(this.ftlUpButton).first();
        await button.click();
    }

    async clickWatch() {
        const button = await this.page.locator(this.watchButton).first();
        await button.click();
    }

    async clickStop() {
        const button = await this.page.locator(this.stopButton).first();
        if (await button.count() > 0) {
            await button.click();
        }
    }

    async getLogOutput() {
        const logArea = await this.page.locator(this.logOutput).first();
        return await logArea.textContent();
    }

    async getCommandOutput() {
        const outputArea = await this.page.locator(this.commandOutput).first();
        return await outputArea.textContent();
    }

    async getProcessStatus() {
        const statusArea = await this.page.locator(this.processStatus).first();
        return await statusArea.textContent();
    }

    async waitForPolling(expectedStatus = null, timeout = 5000) {
        // If expecting a specific status, use HTMXHelpers.waitForPollingUpdate
        if (expectedStatus) {
            return await HTMXHelpers.waitForPollingUpdate(this.page, expectedStatus, {
                timeout: timeout,
                projectPath: this.page.context().projectPath
            });
        }
        // Otherwise just wait for HTMX to settle
        return await HTMXHelpers.waitForSettle(this.page, {
            timeout: timeout,
            projectPath: this.page.context().projectPath
        });
    }
}

module.exports = DashboardPage;