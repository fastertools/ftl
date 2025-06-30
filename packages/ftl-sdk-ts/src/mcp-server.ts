import { Tool } from './tool.js';
import { ToolError, JsonRpcRequest, JsonRpcResponse, JsonRpcError } from './types.js';

/**
 * MCP initialization parameters
 */
interface InitializeParams {
  protocolVersion?: string;
  capabilities?: Record<string, any>;
  clientInfo?: {
    name: string;
    version?: string;
  };
}

/**
 * MCP tools/call parameters
 */
interface ToolsCallParams {
  name: string;
  arguments?: any;
}

/**
 * MCP Server implementation
 */
export class McpServer {
  private tool: Tool;

  constructor(tool: Tool) {
    this.tool = tool;
  }

  /**
   * Handle a JSON-RPC request
   */
  handleRequest(request: JsonRpcRequest): JsonRpcResponse {
    const requestId = request.id ?? null;
    
    try {
      const result = this.processRequest(request);
      
      return {
        jsonrpc: '2.0',
        result,
        id: requestId
      };
    } catch (error: any) {
      return {
        jsonrpc: '2.0',
        error: this.errorToJsonRpc(error),
        id: requestId
      };
    }
  }

  /**
   * Process a request and return the result
   */
  private processRequest(request: JsonRpcRequest): any {
    switch (request.method) {
      case 'initialize':
        return this.handleInitialize(request.params as InitializeParams);
      
      case 'initialized':
        return { status: 'ok' };
      
      case 'tools/list':
        return this.handleToolsList();
      
      case 'tools/call':
        return this.handleToolsCall(request.params as ToolsCallParams);
      
      default:
        throw new McpError(`Method not found: ${request.method}`, -32601);
    }
  }

  /**
   * Handle initialize request
   */
  private handleInitialize(params?: InitializeParams): any {
    return {
      protocolVersion: '2025-03-26',
      serverInfo: {
        name: this.tool.serverName,
        version: this.tool.serverVersion
      },
      capabilities: this.tool.capabilities
    };
  }

  /**
   * Handle tools/list request
   */
  private handleToolsList(): any {
    return {
      tools: [{
        name: this.tool.name,
        description: this.tool.description,
        inputSchema: this.tool.inputSchema
      }]
    };
  }

  /**
   * Handle tools/call request
   */
  private handleToolsCall(params: ToolsCallParams): any {
    if (!params || typeof params !== 'object') {
      throw new McpError('Invalid params', -32602);
    }

    const { name, arguments: args } = params;

    if (name !== this.tool.name) {
      throw new McpError(`Unknown tool: ${name}`, -32602);
    }

    try {
      // Execute the tool (synchronously)
      const result = this.tool.execute(args || {});
      
      // Return the result content
      return result;
    } catch (error: any) {
      if (error instanceof ToolError) {
        throw new McpError(error.message, -32603, {
          code: error.code,
          details: error.details
        });
      }
      throw new McpError(`Tool execution failed: ${error.message}`, -32603);
    }
  }

  /**
   * Convert an error to JSON-RPC format
   */
  private errorToJsonRpc(error: any): JsonRpcError {
    if (error instanceof McpError) {
      return {
        code: error.code,
        message: error.message,
        data: error.data
      };
    }
    
    return {
      code: -32603,
      message: 'Internal error',
      data: error.message
    };
  }
}

/**
 * MCP-specific error
 */
class McpError extends Error {
  code: number;
  data?: any;

  constructor(message: string, code: number, data?: any) {
    super(message);
    this.name = 'McpError';
    this.code = code;
    this.data = data;
  }
}