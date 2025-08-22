const { chromium } = require('playwright');
const TestHelpers = require('../utils/TestHelpers');

class TestFixtures {
    constructor() {
        this.browser = null;
        this.page = null;
        this.context = null;
        this.serverProcess = null;
    }

    async setup(options = {}) {
        // Reset test projects file
        await TestHelpers.resetTestProjectsFile();
        
        // Force server to reload projects from disk
        await this.reloadServerProjects();
        
        // Don't start server - Makefile handles this
        // Server should already be running via 'make test-browser'
        
        // Launch browser
        this.browser = await chromium.launch({ 
            headless: options.headless !== false,
            args: ['--no-sandbox', '--disable-dev-shm-usage']
        });
        
        this.context = await this.browser.newContext();
        this.page = await this.context.newPage();
        
        // Setup console logging if requested
        if (options.logConsole) {
            this.page.on('console', msg => {
                console.log(`BROWSER: ${msg.text()}`);
            });
        }
        
        // Setup error logging
        this.page.on('pageerror', error => {
            console.error(`BROWSER ERROR: ${error.message}`);
        });
        
        // Navigate to homepage to ensure UI reflects server's reloaded state
        await this.page.goto('http://localhost:8080/');
        
        return this.page;
    }

    async reloadServerProjects() {
        // Call the reload endpoint to force server to re-read projects from disk
        try {
            const response = await fetch('http://localhost:8080/htmx/project/reload', {
                method: 'POST'
            });
            if (!response.ok) {
                console.warn('Failed to reload server projects:', response.status);
            } else {
                console.log('Server projects reloaded successfully');
                // Give server time to process the reload
                await new Promise(resolve => setTimeout(resolve, 500));
            }
        } catch (error) {
            console.warn('Failed to reload server projects:', error.message);
        }
    }

    async teardown() {
        if (this.browser) {
            await this.browser.close();
        }
        
        // Don't stop server - Makefile handles this
        
        // Reset test projects file one more time
        await TestHelpers.resetTestProjectsFile();
    }
}

module.exports = TestFixtures;