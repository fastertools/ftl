import { describe, it, expect } from 'vitest';
import { Tool } from '../src/tool.js';

describe('Tool', () => {
  it('should throw an error if name is not implemented', () => {
    class MyTool extends Tool {}
    const tool = new MyTool();
    expect(() => tool.name).toThrow('Tool must implement name getter');
  });

  it('should throw an error if description is not implemented', () => {
    class MyTool extends Tool {
      get name() {
        return 'my-tool';
      }
    }
    const tool = new MyTool();
    expect(() => tool.description).toThrow('Tool must implement description getter');
  });

  it('should return a default input schema', () => {
    class MyTool extends Tool {
      get name() {
        return 'my-tool';
      }
      get description() {
        return 'My tool description';
      }
    }
    const tool = new MyTool();
    expect(tool.inputSchema).toEqual({
      type: 'object',
      properties: {},
      required: []
    });
  });

  it('should throw an error if execute is not implemented', () => {
    class MyTool extends Tool {
      get name() {
        return 'my-tool';
      }
      get description() {
        return 'My tool description';
      }
    }
    const tool = new MyTool();
    expect(() => tool.execute({})).toThrow('Tool must implement execute method');
  });

  it('should return a server name based on the tool name', () => {
    class MyTool extends Tool {
      get name() {
        return 'my-tool';
      }
      get description() {
        return 'My tool description';
      }
    }
    const tool = new MyTool();
    expect(tool.serverName).toBe('ftl-my-tool');
  });

  it('should return a default server version', () => {
    class MyTool extends Tool {
        get name() {
            return 'my-tool';
        }
        get description() {
            return 'My tool description';
        }
    }
    const tool = new MyTool();
    expect(tool.serverVersion).toBe('0.0.1');
  });

  it('should return default capabilities', () => {
    class MyTool extends Tool {
        get name() {
            return 'my-tool';
        }
        get description() {
            return 'My tool description';
        }
    }
    const tool = new MyTool();
    expect(tool.capabilities).toEqual({
        tools: {}
    });
  });
});
