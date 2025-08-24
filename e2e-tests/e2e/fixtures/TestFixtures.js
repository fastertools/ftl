const { chromium } = require('playwright');
const TestHelpers = require('../utils/TestHelpers');
const TestDataManager = require('../data/TestDataManager');

class TestFixtures {
    constructor() {
        this.browser = null;
        this.page = null;
        this.context = null;
        this.serverProcess = null;
    }

    async setup(options = {}) {
        // Use standardized test data initialization
        const fixtureName = options.fixture || 'single';
        await TestDataManager.initialize(fixtureName, options);
        
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

    async setupWithoutProject(options = {}) {
        // For new-user tests - use empty fixture
        await TestDataManager.initialize('empty');
        
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
        
        // Don't navigate yet - let tests control when to navigate
        
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
        
        // Clean up test data using standardized system
        await TestDataManager.cleanup();
    }
}

module.exports = TestFixtures;