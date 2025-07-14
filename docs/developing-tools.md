# Developing Tools

FTL tools are built using the ftl-mcp SDK, which provides templates and utilities for creating MCP components. The SDK supports multiple languages and provides a consistent interface for building tools.

## Tool Interface

When using ftl-mcp templates, tools implement the MCP protocol through language-specific handlers:

### Rust
```rust
use ftl-mcp::*;

// Define your tool with ftl-mcp macros
create_handler!(
    tools: get_tools,
    resources: get_resources,
    prompts: get_prompts
);
```

### TypeScript/JavaScript
```typescript
import { createTool, createResource, createPrompt } from 'ftl-mcp';

// Export your MCP features
export const tools = [/* your tools */];
export const resources = [/* your resources */];
export const prompts = [/* your prompts */];
```

The ftl-mcp SDK handles:
- Protocol compliance with MCP specification
- JSON-RPC request/response handling
- Input validation using JSON Schema
- Error handling and reporting

## Tool Responses

Tools return responses in MCP-compliant formats:

### Success Responses
- Text content: Plain string responses
- JSON content: Structured data responses
- Mixed content: Arrays of content items

### Error Handling
The ftl-mcp SDK provides standard error types:
- Invalid parameters: Schema validation failures
- Execution errors: Runtime failures during tool execution
- Internal errors: Unexpected errors in the tool logic

## Creating Components

FTL uses ftl-mcp templates to scaffold new components:

```bash
# Create a new TypeScript component
ftl add my-tool --language typescript

# Create a new Rust component
ftl add my-tool --language rust
```

The templates provide:
- Pre-configured build system (Makefile)
- Language-specific project structure
- MCP protocol implementation
- Example tool implementations
- Testing setup

For more details on the ftl-mcp SDK and its features, visit the [ftl-mcp repository](https://github.com/fastertools/ftl-mcp).
