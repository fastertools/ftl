import { describe, it, expect } from 'vitest';
import { Tool, ToolResult } from '../dist/index.js';

describe('Tool', () => {
  it('should have abstract methods that must be implemented', () => {
    // Since Tool is abstract, we need to create a concrete implementation
    class MyTool extends Tool {
      get name(): string {
        return 'my-tool';
      }
      get description(): string {
        return 'My tool description';
      }
      execute(args: any): ToolResult {
        return ToolResult.text('test');
      }
    }
    
    const tool = new MyTool();
    expect(tool.name).toBe('my-tool');
    expect(tool.description).toBe('My tool description');
  });

  it('should return a default input schema', () => {
    class MyTool extends Tool {
      get name(): string {
        return 'my-tool';
      }
      get description(): string {
        return 'My tool description';
      }
      execute(args: any): ToolResult {
        return ToolResult.text('test');
      }
    }
    const tool = new MyTool();
    expect(tool.inputSchema).toEqual({
      type: 'object',
      properties: {},
      required: []
    });
  });

  it('should return a server name based on the tool name', () => {
    class MyTool extends Tool {
      get name(): string {
        return 'my-tool';
      }
      get description(): string {
        return 'My tool description';
      }
      execute(args: any): ToolResult {
        return ToolResult.text('test');
      }
    }
    const tool = new MyTool();
    expect(tool.serverName).toBe('ftl-my-tool');
  });

  it('should return a default server version', () => {
    class MyTool extends Tool {
      get name(): string {
        return 'my-tool';
      }
      get description(): string {
        return 'My tool description';
      }
      execute(args: any): ToolResult {
        return ToolResult.text('test');
      }
    }
    const tool = new MyTool();
    expect(tool.serverVersion).toBe('0.0.1');
  });

  it('should return default capabilities', () => {
    class MyTool extends Tool {
      get name(): string {
        return 'my-tool';
      }
      get description(): string {
        return 'My tool description';
      }
      execute(args: any): ToolResult {
        return ToolResult.text('test');
      }
    }
    const tool = new MyTool();
    expect(tool.capabilities).toEqual({
      tools: {}
    });
  });

  it('should allow custom input schema', () => {
    class MyTool extends Tool {
      get name(): string {
        return 'my-tool';
      }
      get description(): string {
        return 'My tool description';
      }
      get inputSchema() {
        return {
          type: 'object',
          properties: {
            input: { type: 'string' }
          },
          required: ['input']
        };
      }
      execute(args: any): ToolResult {
        return ToolResult.text('test');
      }
    }
    const tool = new MyTool();
    expect(tool.inputSchema).toEqual({
      type: 'object',
      properties: {
        input: { type: 'string' }
      },
      required: ['input']
    });
  });
});