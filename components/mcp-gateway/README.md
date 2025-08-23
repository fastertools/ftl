# FTL MCP Gateway

A WebAssembly-based Model Context Protocol (MCP) server that routes tool requests to individual tool components within the Spin framework. Built with Rust and compiled to WebAssembly for secure, sandboxed execution.

## Overview

The MCP Gateway provides a standardized MCP-compliant interface for accessing multiple tool components. It handles protocol negotiation, tool discovery, argument validation, and request routing through Spin's internal networking.

Key features:
- Full MCP JSON-RPC protocol implementation
- Dynamic tool discovery from configured components
- Optional JSON Schema argument validation
- Parallel metadata fetching for optimal performance
- Comprehensive error handling and CORS support

## Architecture

```
MCP Client → JSON-RPC → MCP Gateway → Tool Component (WASM)
                           ↓
                    [Tool Discovery]
                    [Validation]
                    [Routing]
```

## Configuration

Configure the gateway using Spin variables:

```toml
[variables]
component_names = { default = "example-component" }
validate_arguments = { default = "true" }

[component.mcp-gateway.variables]
component_names = "{{ component_names }}"
validate_arguments = "{{ validate_arguments }}"
```

- `component_names`: Comma-separated list of component names that provide tools
- `validate_arguments`: Enable/disable JSON Schema validation of tool arguments

## Protocol Implementation

### Supported Methods

- `initialize` - Establishes protocol version and capabilities
- `initialized` - Notification (no response)
- `tools/list` - Returns metadata for all configured tools
- `tools/call` - Executes a specific tool with arguments
- `ping` - Health check

### Request Flow

1. **Tool Discovery**: Gateway fetches metadata from all configured components in parallel
2. **Name Resolution**: Component names are converted from snake_case to kebab-case
3. **Validation**: Arguments are validated against tool's JSON Schema (if enabled)
4. **Routing**: Requests are forwarded to `http://{component-name}.spin.internal/`
5. **Response**: Tool execution results are returned in MCP-compliant format

## Tool Component Requirements

Each tool component must:

1. Respond to GET requests with metadata:
```json
{
  "name": "tool_name",
  "title": "Human Readable Name",
  "description": "What this tool does",
  "inputSchema": { /* JSON Schema */ }
}
```

2. Respond to POST requests with tool execution:
```json
{
  "content": [
    {
      "type": "text",
      "text": "Tool output"
    }
  ]
}
```

## Error Handling

The gateway returns JSON-RPC error responses for:
- Invalid protocol versions
- Unknown methods
- Missing tools
- Validation failures
- Tool execution errors

Standard error codes:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## Performance

- Parallel metadata fetching across all configured components
- Efficient routing through Spin's internal networking stack
- Optional argument validation for performance-sensitive scenarios
- WebAssembly-based execution with minimal overhead

## Usage Example

```bash
# Initialize the connection
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": {
        "name": "example-client",
        "version": "1.0.0"
      }
    },
    "id": 1
  }'

# List available tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":2}'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {"message": "Hello"}
    },
    "id": 3
  }'
```

## Development

### Requirements
- Rust toolchain with `wasm32-wasip1` target
- Spin CLI (v2.0+)

### Building
```bash
cargo build --target wasm32-wasip1 --release
```

### Testing
```bash
cd tests
./run_tests.sh
```

### Architecture
Built with:
- Rust and Spin SDK for WebAssembly runtime
- `jsonschema` crate for argument validation
- Async/await for concurrent component communication
- `serde` for JSON serialization/deserialization