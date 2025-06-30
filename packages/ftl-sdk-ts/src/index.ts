import { AutoRouter } from 'itty-router';
import { McpServer } from './mcp-server.js';
import { Tool } from './tool.js';
import { ToolResult, ToolError } from './types.js';

// Define FetchEvent type for WebWorker environment
interface FetchEvent extends Event {
  request: Request;
  respondWith(response: Response | Promise<Response>): void;
}

/**
 * Register an FTL tool and set up the fetch event listener
 * @param tool - The tool implementation
 */
export function ftlTool(tool: Tool): void {
  const server = new McpServer(tool);
  const router = AutoRouter();

  // MCP endpoint
  router.post('/mcp', async (request: Request) => {
    try {
      // In Spin, we use async/await with request.text()
      const bodyText = await request.text();
      const jsonRpcRequest = JSON.parse(bodyText);
      
      // Handle the request
      const response = server.handleRequest(jsonRpcRequest);
      
      return new Response(JSON.stringify(response), {
        headers: { 'Content-Type': 'application/json' }
      });
    } catch (error: any) {
      return new Response(JSON.stringify({
        jsonrpc: '2.0',
        error: {
          code: -32700,
          message: 'Parse error',
          data: error.message
        },
        id: null
      }), {
        status: 400,
        headers: { 'Content-Type': 'application/json' }
      });
    }
  });

  // Health check endpoint
  router.get('/health', () => new Response('OK'));
  
  // Info endpoint
  router.get('/', () => new Response(JSON.stringify({
    name: tool.name,
    version: tool.version || '0.1.0',
    description: tool.description,
    mcp_endpoint: '/mcp'
  }), {
    headers: { 'Content-Type': 'application/json' }
  }));

  // 404 handler
  router.all('*', () => new Response('Not Found', { status: 404 }));
  
  // Register fetch event listener
  addEventListener('fetch', (event: Event) => {
    const fetchEvent = event as FetchEvent;
    fetchEvent.respondWith(router.fetch(fetchEvent.request));
  });
}

// Re-export everything
export { Tool, ToolResult, ToolError, McpServer };

// Default export for convenience
export default {
  ftlTool,
  Tool,
  ToolResult,
  ToolError,
  McpServer
};