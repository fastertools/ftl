import { ToolResult, ToolError } from './types.js';

/**
 * JSON Schema type definition
 */
export interface JsonSchema {
  type: string;
  properties?: Record<string, any>;
  required?: string[];
  [key: string]: any;
}

/**
 * Server capabilities
 */
export interface ServerCapabilities {
  tools?: Record<string, any>;
  [key: string]: any;
}

/**
 * Base class for FTL tools
 */
export abstract class Tool {
  /**
   * The name of the tool (used in MCP tool calls)
   */
  abstract get name(): string;

  /**
   * Human-readable description of what the tool does
   */
  abstract get description(): string;

  /**
   * JSON schema for the tool's input parameters
   */
  get inputSchema(): JsonSchema {
    return {
      type: 'object',
      properties: {},
      required: []
    };
  }

  /**
   * Execute the tool with the provided arguments
   * @param args - The arguments object
   * @returns The tool result
   * @throws ToolError if the tool execution fails
   */
  abstract execute(args: any): ToolResult | Promise<ToolResult>;

  /**
   * Optional: Server name (defaults to tool name)
   */
  get serverName(): string {
    return `ftl-${this.name}`;
  }

  /**
   * Optional: Server version
   */
  get serverVersion(): string {
    return '0.0.1';
  }

  /**
   * Optional: Additional server capabilities
   */
  get capabilities(): ServerCapabilities {
    return {
      tools: {}
    };
  }

  /**
   * Optional: Tool version (for display purposes)
   */
  get version(): string | undefined {
    return undefined;
  }
}