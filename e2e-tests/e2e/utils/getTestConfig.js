const fs = require('fs');
const path = require('path');

/**
 * Get test configuration including dynamic base URL
 */
function getTestConfig() {
    // Try to load dynamic config
    const testConfigPath = path.join(__dirname, '../../.test-config.json');
    if (fs.existsSync(testConfigPath)) {
        const config = JSON.parse(fs.readFileSync(testConfigPath, 'utf8'));
        return {
            baseURL: config.baseURL || 'http://localhost:8080',
            port: config.port || 8080
        };
    }
    
    // Fallback to defaults
    return {
        baseURL: process.env.TEST_BASE_URL || 'http://localhost:8080',
        port: process.env.TEST_PORT || 8080
    };
}

module.exports = { getTestConfig };