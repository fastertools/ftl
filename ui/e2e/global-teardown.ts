import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';
import TestHelpers from './utils/test-helpers';

async function globalTeardown(): Promise<void> {
    console.log('🧹 Starting global test teardown...');
    
    // Kill the test server
    TestHelpers.stopConsoleServer();
    console.log('✅ Test server stopped');
    
    // Clean up test config
    const testConfigFile = path.join(__dirname, '.test-config.json');
    if (fs.existsSync(testConfigFile)) {
        fs.unlinkSync(testConfigFile);
    }
    
    // Clean up test projects file
    if (fs.existsSync(TestHelpers.TEST_PROJECTS_FILE)) {
        fs.unlinkSync(TestHelpers.TEST_PROJECTS_FILE);
        console.log('✅ Test projects file cleaned');
    }
    
    // Final cleanup of any orphaned processes
    try {
        execSync('pkill -f chromium || true', { stdio: 'ignore' });
    } catch {
        // Ignore errors
    }
    
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