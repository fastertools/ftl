import { describe, it, expect } from 'vitest';
import { ToolResult, ToolError } from '../dist/index.js';

describe('ToolResult', () => {
  it('should create a text result', () => {
    const result = ToolResult.text('Hello, world!');
    expect(result.content).toEqual([
      { type: 'text', text: 'Hello, world!' }
    ]);
  });

  it('should create a JSON result', () => {
    const data = { foo: 'bar', count: 42 };
    const result = ToolResult.json(data);
    expect(result.content).toEqual([
      { type: 'text', text: JSON.stringify(data, null, 2) }
    ]);
  });

  it('should create a multi-part result', () => {
    const parts = [
      { type: 'text', text: 'Part 1' },
      { type: 'text', text: 'Part 2' },
      { type: 'data', data: { key: 'value' } }
    ];
    const result = ToolResult.multi(parts);
    expect(result.content).toEqual(parts);
  });
});

describe('ToolError', () => {
  it('should create a basic error', () => {
    const error = new ToolError('Something went wrong');
    expect(error.message).toBe('Something went wrong');
    expect(error.code).toBe('TOOL_ERROR');
    expect(error.details).toBeNull();
    expect(error.name).toBe('ToolError');
  });

  it('should create an error with custom code and details', () => {
    const details = { field: 'input', reason: 'too long' };
    const error = new ToolError('Validation failed', 'VALIDATION_ERROR', details);
    expect(error.message).toBe('Validation failed');
    expect(error.code).toBe('VALIDATION_ERROR');
    expect(error.details).toEqual(details);
  });

  it('should create an invalid arguments error', () => {
    const error = ToolError.invalidArguments('Missing required field: input');
    expect(error.message).toBe('Missing required field: input');
    expect(error.code).toBe('INVALID_ARGUMENTS');
    expect(error.details).toBeNull();
  });

  it('should create an execution error', () => {
    const error = ToolError.executionError('API call failed');
    expect(error.message).toBe('API call failed');
    expect(error.code).toBe('EXECUTION_ERROR');
    expect(error.details).toBeNull();
  });

  it('should create an execution error with details', () => {
    const details = { statusCode: 500, response: 'Internal Server Error' };
    const error = ToolError.executionError('API call failed', details);
    expect(error.message).toBe('API call failed');
    expect(error.code).toBe('EXECUTION_ERROR');
    expect(error.details).toEqual(details);
  });
});