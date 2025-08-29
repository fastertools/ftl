import TestHelpers from './utils/test-helpers';
import ServerManager from './utils/server-manager';
import * as fs from 'fs';
import * as path from 'path';

interface TestConfig {
    baseURL: string;
    serverStartTime: number;
}

async function globalSetup(): Promise<TestConfig> {
    console.log('🚀 Starting global test setup...');
    
    // Clean up orphaned browsers from previous runs
    ServerManager.cleanupOrphanedBrowsers();
    
    try {
        // Start server with dynamic port using TestHelpers
        const baseURL = await TestHelpers.startServer({
            useTestProjects: true,
            timeout: 30000,
            retries: 3
        });
        
        console.log(`✅ Test server ready at ${baseURL}`);
        
        // Store server info for tests to use
        const testConfig: TestConfig = {
            baseURL,
            serverStartTime: Date.now()
        };
        
        // Save config for tests
        fs.writeFileSync(
            path.join(__dirname, '.test-config.json'),
            JSON.stringify(testConfig, null, 2)
        );
        
        return testConfig;
        
    } catch (error) {
        console.error('❌ Failed to start test server:', error);
        throw error;
    }
}

export = globalSetup;