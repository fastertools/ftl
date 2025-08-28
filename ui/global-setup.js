const ServerManager = require('./e2e/utils/ServerManager');
const fs = require('fs');
const path = require('path');

async function globalSetup() {
    console.log('🚀 Starting global test setup...');
    
    // Clean up orphaned browsers from previous runs
    ServerManager.cleanupOrphanedBrowsers();
    
    // Create server manager
    const serverManager = new ServerManager();
    
    try {
        // Start server with dynamic port
        const baseURL = await serverManager.startServer({
            useTestProjects: true,
            timeout: 30000,
            retries: 3
        });
        
        console.log(`✅ Test server ready at ${baseURL}`);
        
        // Store server info for tests to use
        const testConfig = {
            baseURL,
            port: serverManager.getPort(),
            serverStartTime: Date.now()
        };
        
        // Save config for tests
        fs.writeFileSync(
            path.join(__dirname, '.test-config.json'),
            JSON.stringify(testConfig, null, 2)
        );
        
        // Store server manager globally for teardown
        global.__SERVER_MANAGER__ = serverManager;
        
        return testConfig;
        
    } catch (error) {
        console.error('❌ Failed to start test server:', error);
        throw error;
    }
}

module.exports = globalSetup;