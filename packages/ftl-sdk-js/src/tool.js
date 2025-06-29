/**
 * Base class for FTL tools
 * @abstract
 */
export class Tool {
  /**
   * The name of the tool (used in MCP tool calls)
   * @type {string}
   */
  get name() {
    throw new Error('Tool must implement name getter');
  }

  /**
   * Human-readable description of what the tool does
   * @type {string}
   */
  get description() {
    throw new Error('Tool must implement description getter');
  }

  /**
   * JSON schema for the tool's input parameters
   * @returns {Object} JSON Schema object
   */
  get inputSchema() {
    return {
      type: 'object',
      properties: {},
      required: []
    };
  }

  /**
   * Execute the tool with the provided arguments
   * @param {Object} args - The arguments object
   * @returns {import('./types.js').ToolResult} The tool result
   * @throws {import('./types.js').ToolError} If the tool execution fails
   */
  execute(args) {
    throw new Error('Tool must implement execute method');
  }

  /**
   * Optional: Server name (defaults to tool name)
   * @type {string}
   */
  get serverName() {
    return `ftl-${this.name}`;
  }

  /**
   * Optional: Server version
   * @type {string}
   */
  get serverVersion() {
    return '0.0.1';
  }

  /**
   * Optional: Additional server capabilities
   * @returns {Object} Capabilities object
   */
  get capabilities() {
    return {
      tools: {}
    };
  }
}