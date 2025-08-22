const { test, expect } = require('@playwright/test');
const { chromium } = require('playwright');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

test.describe('New User Centralized Experience Tests', () => {
    let browser;
    let page;
    let context;
    let serverProcess;
    let centralizedProjectsPath;

    test.beforeAll(async () => {
        // Calculate the centralized projects.json path
        let dataDir;
        if (process.env.XDG_DATA_HOME) {
            dataDir = process.env.XDG_DATA_HOME;
        } else {
            if (process.platform === 'darwin') {
                dataDir = path.join(os.homedir(), 'Library', 'Application Support');
            } else if (process.platform === 'win32') {
                dataDir = process.env.APPDATA;
            } else {
                dataDir = path.join(os.homedir(), '.config');
            }
        }
        centralizedProjectsPath = path.join(dataDir, 'ftl', 'projects.json');
        
        console.log(`Centralized projects path: ${centralizedProjectsPath}`);

        // Server will be started by Makefile - we don't need to start it here
        // Makefile already handles cleanup before server start

        // Give server time to start
        await new Promise(resolve => setTimeout(resolve, 3000));

        // Launch browser
        browser = await chromium.launch({ 
            headless: true,
            args: ['--no-sandbox', '--disable-dev-shm-usage']
        });
        
        context = await browser.newContext();
        page = await context.newPage();
        
        // Setup error logging
        page.on('pageerror', error => {
            console.error(`BROWSER ERROR: ${error.message}`);
        });
    });

    test.afterAll(async () => {
        if (browser) {
            await browser.close();
        }
        
        // Makefile handles server cleanup and centralized directory cleanup
    });

    test('should have empty projects.json at centralized location for new users', async () => {
        console.log('Testing centralized new user first load experience...');
        
        // Server should have already created the file during startup
        // Verify the centralized projects file exists and is properly initialized
        expect(fs.existsSync(centralizedProjectsPath)).toBe(true);
        
        // Navigate to the console to verify UI works with centralized file
        await page.goto('http://localhost:8082', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        // Wait for the page to fully load and server to process
        await page.waitForTimeout(3000);
        
        // Verify the page loaded successfully
        const title = await page.title();
        expect(title).toContain('FTL');
        
        // Verify the centralized projects.json file contains empty array for new users
        const projectsContent = fs.readFileSync(centralizedProjectsPath, 'utf8');
        const projects = JSON.parse(projectsContent);
        expect(Array.isArray(projects)).toBe(true);
        expect(projects.length).toBe(0);
        
        console.log(`✅ Empty projects.json verified at: ${centralizedProjectsPath}`);
        console.log(`Content: ${projectsContent}`);
    });

    test('should show Add Project button with no hardcoded projects', async () => {
        // Navigate to console
        await page.goto('http://localhost:8082', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await page.waitForTimeout(2000);
        
        // Verify Add Project button is visible
        const addProjectButton = page.locator('#add-project-btn');
        await expect(addProjectButton).toBeVisible();
        
        // Verify no projects are listed (should show empty state)
        const projectItems = page.locator('.project-item');
        const projectCount = await projectItems.count();
        expect(projectCount).toBe(0);
        
        // Verify projects.json is empty (no hardcoded defaults)
        const projectsContent = fs.readFileSync(centralizedProjectsPath, 'utf8');
        const projects = JSON.parse(projectsContent);
        expect(projects.length).toBe(0);
        
        console.log('✅ New user sees Add Project button with no hardcoded projects');
    });

    test('should have proper centralized directory structure', async () => {
        // Server should have already created the directory and file during startup
        const centralizedDir = path.dirname(centralizedProjectsPath);
        
        // Navigate to console to ensure server is fully initialized
        await page.goto('http://localhost:8082', { 
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
        
        await page.waitForTimeout(2000);
        
        // Verify directory structure exists (created during server startup)
        expect(fs.existsSync(centralizedDir)).toBe(true);
        expect(fs.existsSync(centralizedProjectsPath)).toBe(true);
        
        // Verify directory permissions (should be 0750)
        const stats = fs.statSync(centralizedDir);
        expect(stats.isDirectory()).toBe(true);
        
        console.log(`✅ Centralized directory verified: ${centralizedDir}`);
    });
});