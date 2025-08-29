import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';
import TestHelpers from './utils/test-helpers';

// Extend global interface to type the server manager
declare global {
    var __SERVER_MANAGER__: any;
}

async function globalTeardown(): Promise<void> {
    console.log('🧹 Starting global test teardown...');
    
    // Kill the test server
    if (global.__SERVER_MANAGER__) {
        global.__SERVER_MANAGER__.killServer();
        console.log('✅ Test server stopped');
    }
    
    // Clean up test config
    if (fs.existsSync(TestHelpers.TEST_CONFIG_FILE)) {
        fs.unlinkSync(TestHelpers.TEST_CONFIG_FILE);
    }
    
    // Clean up test projects file
    if (fs.existsSync(TestHelpers.TEST_PROJECTS_FILE)) {
        fs.unlinkSync(TestHelpers.TEST_PROJECTS_FILE);
        console.log('✅ Test projects file cleaned');
    }
    
    // Final cleanup of any orphaned processes
    const { default: ServerManager } = await import('./utils/server-manager');
    ServerManager.cleanupOrphanedBrowsers();
    
    // Kill any remaining FTL processes
    try {
        execSync('pkill -f "ftl console" || true', { stdio: 'ignore' });
        execSync('pkill -f "ftl dev console" || true', { stdio: 'ignore' });
        console.log('✅ Cleaned up orphaned FTL processes');
    } catch (error) {
        // Ignore errors
    }
    
    console.log('✅ Global teardown complete');
}

export = globalTeardown;