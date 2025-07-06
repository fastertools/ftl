import { describe, it, expect } from 'vitest';
import { tools } from '../src/features.js';

describe('{{project-name}} MCP Handler', () => {
    describe('Tools', () => {
        it('should export at least one tool', () => {
            expect(tools).toBeDefined();
            expect(Object.keys(tools).length).toBeGreaterThan(0);
        });

        it('should have the {{project-name | snake_case}} tool', () => {
            expect(tools['{{project-name | snake_case}}']).toBeDefined();
            expect(tools['{{project-name | snake_case}}'].description).toBe('{{project-description}}');
        });

        it('should handle valid input for {{project-name | snake_case}} tool', async () => {
            const tool = tools['{{project-name | snake_case}}'];
            expect(tool.handler).toBeDefined();
            
            const result = await tool.handler({ input: 'test input' });
            expect(result).toBeDefined();
            expect(typeof result).toBe('string');
            expect(result).toContain('test input');
        });

        it('should handle missing input gracefully', async () => {
            const tool = tools['{{project-name | snake_case}}'];
            
            const result = await tool.handler({});
            expect(result).toBeDefined();
            expect(typeof result).toBe('string');
        });
    });
});