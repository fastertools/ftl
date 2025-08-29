import { describe, it, expect, beforeAll } from 'vitest';
import { MCPClient } from './mcp-client';
import * as path from 'path';
import * as fs from 'fs';

describe('MCPClient', () => {
    const projectRoot = path.resolve(__dirname, '..', '..', '..');
    const ftlBinaryPath = path.join(projectRoot, 'bin', 'ftl');
    let mcpClient: MCPClient;

    beforeAll(() => {
        // Verify FTL binary exists
        expect(fs.existsSync(ftlBinaryPath)).toBe(true);
        
        mcpClient = new MCPClient({
            ftlBinaryPath,
            testMode: true,
            validateInputs: false, // Skip validation for debugging
            timeout: 10000
        });
    });

    it('should have correct FTL binary path configured', () => {
        expect((mcpClient as any).ftlBinaryPath).toBe(ftlBinaryPath);
        expect((mcpClient as any).testMode).toBe(true);
    });

    it('should be able to call MCP health check', async () => {
        const result = await mcpClient.callToolUnsafe('mcp-server__health_check', {});
        
        console.log('Health check result:', result);
        
        // Log raw output for debugging
        if (result.raw_output) {
            console.log('Raw output:', result.raw_output);
        }
        
        // Should not immediately fail (even if MCP tools aren't implemented yet)
        expect(result).toHaveProperty('success');
        expect(result).toHaveProperty('error');
    });

    it('should be able to call MCP init tool', async () => {
        const result = await mcpClient.callToolUnsafe('mcp-server__init', {
            project_name: 'test-debug',
            language: 'rust',
            base_path: '/tmp'
        });
        
        console.log('Init tool result:', result);
        
        // Log raw output for debugging
        if (result.raw_output) {
            console.log('Raw output:', result.raw_output);
        }
        
        // Should not immediately fail (even if MCP tools aren't implemented yet)
        expect(result).toHaveProperty('success');
        expect(result).toHaveProperty('error');
    });

    it('should pass correct environment variables', async () => {
        // This test will help us see what environment is actually being passed
        const result = await mcpClient.callToolUnsafe('mcp-server__health_check', {});
        
        // Log the result to see what the MCP server is actually receiving
        console.log('Environment test result:', JSON.stringify(result, null, 2));
        
        expect(result).toBeDefined();
    });
});