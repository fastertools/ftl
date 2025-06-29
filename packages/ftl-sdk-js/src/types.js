/**
 * Result returned by a tool execution
 */
export class ToolResult {
  /**
   * @param {Array<{type: string, text?: string, data?: any}>} content - Content blocks
   */
  constructor(content) {
    this.content = content;
  }

  /**
   * Create a text result
   * @param {string} text - The text content
   * @returns {ToolResult}
   */
  static text(text) {
    return new ToolResult([{ type: 'text', text }]);
  }

  /**
   * Create a JSON result
   * @param {any} data - The data to return
   * @returns {ToolResult}
   */
  static json(data) {
    return new ToolResult([{ 
      type: 'text', 
      text: JSON.stringify(data, null, 2) 
    }]);
  }

  /**
   * Create a multi-part result
   * @param {Array<{type: string, text?: string, data?: any}>} parts - Content parts
   * @returns {ToolResult}
   */
  static multi(parts) {
    return new ToolResult(parts);
  }
}

/**
 * Error thrown by tool execution
 */
export class ToolError extends Error {
  /**
   * @param {string} message - Error message
   * @param {string} [code] - Error code
   * @param {any} [details] - Additional error details
   */
  constructor(message, code = 'TOOL_ERROR', details = null) {
    super(message);
    this.name = 'ToolError';
    this.code = code;
    this.details = details;
  }

  /**
   * Create an invalid arguments error
   * @param {string} message - Error message
   * @returns {ToolError}
   */
  static invalidArguments(message) {
    return new ToolError(message, 'INVALID_ARGUMENTS');
  }

  /**
   * Create an execution error
   * @param {string} message - Error message
   * @param {any} [details] - Error details
   * @returns {ToolError}
   */
  static executionError(message, details) {
    return new ToolError(message, 'EXECUTION_ERROR', details);
  }
}

/**
 * JSON-RPC request
 * @typedef {Object} JsonRpcRequest
 * @property {string} jsonrpc - Always "2.0"
 * @property {string} method - Method name
 * @property {any} [params] - Method parameters
 * @property {string|number|null} [id] - Request ID
 */

/**
 * JSON-RPC response
 * @typedef {Object} JsonRpcResponse
 * @property {string} jsonrpc - Always "2.0"
 * @property {any} [result] - Success result
 * @property {Object} [error] - Error object
 * @property {string|number|null} id - Request ID
 */

/**
 * JSON-RPC error
 * @typedef {Object} JsonRpcError
 * @property {number} code - Error code
 * @property {string} message - Error message
 * @property {any} [data] - Additional error data
 */