import { describe, it, expect, beforeAll } from 'vitest';
import { MCPToolValidator, validateMCPToolInput } from './mcp-tool-validator';

describe('MCPToolValidator', () => {
    let validator: MCPToolValidator;

    beforeAll(() => {
        validator = new MCPToolValidator({ verbose: true });
    });

    describe('server_start tool validation', () => {
        it('should validate correct server_start input', async () => {
            const input = {
                use_test_projects: true,
                project_base: "/Users/coreyryan/data/mashh/ftl-cli/ui/e2e/.e2e-projects",
                timeout_ms: 30000,
                ftl_binary_path: "/Users/coreyryan/data/mashh/ftl-cli/bin/ftl"
            };

            const result = await validator.validateToolInput('mcp-server__server_start', input, { verbose: true });
            
            if (!result.valid) {
                console.error('Input:', JSON.stringify(input, null, 2));
                console.error('Validation errors:', result.errors);
                console.error('Validation warnings:', result.warnings);
            }
            
            expect(result.valid).toBe(true);
        });

        it('should validate minimal server_start input with defaults', async () => {
            const input = {}; // All fields are optional with defaults

            const result = await validator.validateToolInput('mcp-server__server_start', input);
            
            if (!result.valid) {
                console.error('Validation errors:', result.errors);
            }
            
            expect(result.valid).toBe(true);
        });

        it('should reject invalid timeout_ms', async () => {
            const input = {
                timeout_ms: 500 // Below minimum of 1000
            };

            const result = await validator.validateToolInput('mcp-server__server_start', input);
            expect(result.valid).toBe(false);
            expect(result.errors.some(err => err.includes('timeout_ms'))).toBe(true);
        });

        it('should reject invalid port', async () => {
            const input = {
                port: 80 // Below minimum of 1024
            };

            const result = await validator.validateToolInput('mcp-server__server_start', input);
            expect(result.valid).toBe(false);
            expect(result.errors.some(err => err.includes('port'))).toBe(true);
        });
    });

    describe('init tool validation', () => {
        it('should validate correct init input', async () => {
            const input = {
                project_name: "test-project",
                language: "rust",
                base_path: "/Users/coreyryan/data/mashh/ftl-cli/ui/e2e/.e2e-projects"
            };

            const result = await validator.validateToolInput('mcp-server__init', input);
            
            if (!result.valid) {
                console.error('Init validation errors:', result.errors);
                console.error('Init validation warnings:', result.warnings);
            }
            
            expect(result.valid).toBe(true);
        });

        it('should reject init input without project_name', async () => {
            const input = {
                language: "rust"
                // missing project_name
            };

            const result = await validator.validateToolInput('mcp-server__init', input, { verbose: true });
            
            if (result.valid) {
                console.error('UNEXPECTED: Validation passed when it should have failed');
                console.error('Input:', JSON.stringify(input, null, 2));
                console.error('Result:', JSON.stringify(result, null, 2));
            }
            
            expect(result.valid).toBe(false);
            expect(result.errors.some(err => err.includes('project_name'))).toBe(true);
        });

        it('should reject invalid language', async () => {
            const input = {
                project_name: "test-project",
                language: "invalid-language"
            };

            const result = await validator.validateToolInput('mcp-server__init', input);
            expect(result.valid).toBe(false);
            expect(result.errors.some(err => err.includes('language'))).toBe(true);
        });
    });

    describe('schema discovery', () => {
        it('should have schemas for all expected tools', () => {
            const schemas = validator.getAvailableToolSchemas();
            
            const expectedTools = [
                'mcp-server__init',
                'mcp-server__add',
                'mcp-server__build',
                'mcp-server__up',
                'mcp-server__stop',
                'mcp-server__status',
                'mcp-server__logs',
                'mcp-server__server_start',
                'mcp-server__server_stop',
                'mcp-server__server_status'
            ];

            expectedTools.forEach(tool => {
                expect(schemas).toHaveProperty(tool);
                expect(schemas[tool]).toMatch(/^#\w+Input$/);
            });
        });

        it('should reject unknown tools', async () => {
            const result = await validator.validateToolInput('unknown-tool', {});
            expect(result.valid).toBe(false);
            expect(result.errors[0]).toContain('Unknown MCP tool');
        });
    });

    describe('convenience function', () => {
        it('should work via convenience function', async () => {
            const input = {
                use_test_projects: true,
                timeout_ms: 30000
            };

            const result = await validateMCPToolInput('mcp-server__server_start', input);
            expect(result.valid).toBe(true);
        });
    });
});