import TestHelpers from './utils/test-helpers';
import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

interface TestConfig {
    baseURL: string;
    serverStartTime: number;
}

async function globalSetup(): Promise<TestConfig> {
    console.log('🚀 Starting global test setup...');
    
    // Clean up orphaned browsers from previous runs
    try {
        execSync('pkill -f chromium || true', { stdio: 'ignore' });
    } catch {
        // Ignore errors
    }
    
    try {
        // Find available port and start console server
        const port = await TestHelpers.findAvailablePort(8080);
        await TestHelpers.startConsoleServer({
            port: port,
            useTestProjects: true,
            timeout: 30000,
            retries: 3
        });
        
        const baseURL = `http://localhost:${port}`;
        
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