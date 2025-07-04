import { describe, it, expect } from 'vitest';
import { Handler } from '../src/index.js';

describe('Handler', () => {
    describe('listTools', () => {
        it('should return tool metadata', () => {
            const tools = Handler.listTools();
            expect(tools).toHaveLength(1);
            expect(tools[0].name).toBe('{{project-name | snake_case}}');
            expect(tools[0].description).toBe('{{project-description}}');
        });
    });

    describe('callTool', () => {
        it('should process input correctly', () => {
            const args = JSON.stringify({ input: 'test input' });
            const result = Handler.callTool('{{project-name | snake_case}}', args);
            
            expect(result.tag).toBe('text');
            expect(result.val).toContain('test input');
        });

        it('should handle invalid JSON', () => {
            const result = Handler.callTool('{{project-name | snake_case}}', 'invalid json');
            
            expect(result.tag).toBe('error');
            expect(result.val.code).toBe(-32602);
        });

        it('should handle unknown tool', () => {
            const args = JSON.stringify({ input: 'test' });
            const result = Handler.callTool('unknown_tool', args);
            
            expect(result.tag).toBe('error');
            expect(result.val.code).toBe(-32601);
        });
    });
});