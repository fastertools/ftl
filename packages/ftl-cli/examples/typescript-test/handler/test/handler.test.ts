import { describe, it, expect } from 'vitest';
import { Handler } from '../src/index';

describe('Handler', () => {
    describe('listTools', () => {
        it('should return tool metadata', () => {
            const tools = Handler.listTools();
            expect(tools).toHaveLength(1);
            expect(tools[0].name).toBe('typescript_test');
            expect(tools[0].description).toBe('An MCP tool written in TypeScript');
        });
    });

    describe('callTool', () => {
        it('should process input correctly', () => {
            const args = JSON.stringify({ input: 'test input' });
            const result = Handler.callTool('typescript_test', args);
            
            expect(result.tag).toBe('text');
            if (result.tag === 'text') {
                expect(result.val).toContain('test input');
            }
        });

        it('should handle invalid JSON', () => {
            const result = Handler.callTool('typescript_test', 'invalid json');
            
            expect(result.tag).toBe('error');
            if (result.tag === 'error') {
                expect(result.val.code).toBe(-32602);
            }
        });

        it('should handle unknown tool', () => {
            const args = JSON.stringify({ input: 'test' });
            const result = Handler.callTool('unknown_tool', args);
            
            expect(result.tag).toBe('error');
            if (result.tag === 'error') {
                expect(result.val.code).toBe(-32601);
            }
        });
    });
});