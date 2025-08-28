const ServerManager = require('./e2e/utils/ServerManager');
const fs = require('fs');
const path = require('path');

async function globalTeardown() {
    console.log('🧹 Starting global test teardown...');
    
    // Kill the test server
    if (global.__SERVER_MANAGER__) {
        global.__SERVER_MANAGER__.killServer();
        console.log('✅ Test server stopped');
    }
    
    // Clean up test config
    const configPath = path.join(__dirname, '.test-config.json');
    if (fs.existsSync(configPath)) {
        fs.unlinkSync(configPath);
    }
    
    // Clean up test projects file
    const testProjectsPath = path.join(process.cwd(), '.e2e-projects.json');
    if (fs.existsSync(testProjectsPath)) {
        fs.unlinkSync(testProjectsPath);
        console.log('✅ Test projects file cleaned');
    }
    
    // Final cleanup of any orphaned processes
    ServerManager.cleanupOrphanedBrowsers();
    
    // Kill any remaining FTL processes
    try {
        const { execSync } = require('child_process');
        execSync('pkill -f "ftl console" || true', { stdio: 'ignore' });
        execSync('pkill -f "ftl dev console" || true', { stdio: 'ignore' });
        console.log('✅ Cleaned up orphaned FTL processes');
    } catch (error) {
        // Ignore errors
    }
    
    console.log('✅ Global teardown complete');
}

module.exports = globalTeardown;