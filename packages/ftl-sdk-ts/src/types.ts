/**
 * Content block type for tool results
 */
export interface ContentBlock {
  type: string;
  text?: string;
  data?: any;
}

/**
 * Result returned by a tool execution
 */
export class ToolResult {
  content: ContentBlock[];

  constructor(content: ContentBlock[]) {
    this.content = content;
  }

  /**
   * Create a text result
   */
  static text(text: string): ToolResult {
    return new ToolResult([{ type: 'text', text }]);
  }

  /**
   * Create a JSON result
   */
  static json(data: any): ToolResult {
    return new ToolResult([{ 
      type: 'text', 
      text: JSON.stringify(data, null, 2) 
    }]);
  }

  /**
   * Create a multi-part result
   */
  static multi(parts: ContentBlock[]): ToolResult {
    return new ToolResult(parts);
  }
}

/**
 * Error thrown by tool execution
 */
export class ToolError extends Error {
  code: string;
  details: any;

  constructor(message: string, code: string = 'TOOL_ERROR', details: any = null) {
    super(message);
    this.name = 'ToolError';
    this.code = code;
    this.details = details;
  }

  /**
   * Create an invalid arguments error
   */
  static invalidArguments(message: string): ToolError {
    return new ToolError(message, 'INVALID_ARGUMENTS');
  }

  /**
   * Create an execution error
   */
  static executionError(message: string, details?: any): ToolError {
    return new ToolError(message, 'EXECUTION_ERROR', details);
  }
}

/**
 * JSON-RPC request
 */
export interface JsonRpcRequest {
  jsonrpc: string;
  method: string;
  params?: any;
  id?: string | number | null;
}

/**
 * JSON-RPC response
 */
export interface JsonRpcResponse {
  jsonrpc: string;
  result?: any;
  error?: JsonRpcError;
  id: string | number | null;
}

/**
 * JSON-RPC error
 */
export interface JsonRpcError {
  code: number;
  message: string;
  data?: any;
}