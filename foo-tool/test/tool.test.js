import { FooTool } from '../src/index.js';

describe('FooTool', () => {
  let tool;

  beforeEach(() => {
    tool = new FooTool();
  });

  test('should have correct metadata', () => {
    expect(tool.name).toBe('foo-tool');
    expect(tool.description).toBe('A tool that does foo');
  });

  test('should define input schema', () => {
    const schema = tool.inputSchema;
    expect(schema.type).toBe('object');
    expect(schema.properties.input).toBeDefined();
    expect(schema.required).toContain('input');
  });

  test('should process valid input', () => {
    const result = tool.execute({ input: 'test' });
    expect(result.content).toHaveLength(1);
    expect(result.content[0].type).toBe('text');
    expect(result.content[0].text).toContain('test');
  });

  test('should throw error for missing input', () => {
    expect(() => tool.execute({})).toThrow('Input is required');
  });
});