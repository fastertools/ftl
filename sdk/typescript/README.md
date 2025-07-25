# ftl-sdk (TypeScript)

TypeScript SDK for building Model Context Protocol (MCP) tools that compile to WebAssembly.

## Installation

```bash
npm install ftl-sdk
```

## Overview

This SDK provides:
- TypeScript type definitions for the MCP protocol
- Zero-dependency `createTools` helper for building multiple tools per component
- Seamless integration with Zod for schema validation
- Full compatibility with Spin WebAssembly components

## Quick Start

### Using the `createTools` Helper (Recommended)

The SDK includes a `createTools` helper that handles the MCP protocol for you:

```typescript
import { createTools, ToolResponse } from 'ftl-sdk'
import { z } from 'zod'

// Define your input schemas using Zod
const EchoSchema = z.object({
  message: z.string().describe('The message to echo')
})

const ReverseSchema = z.object({
  text: z.string().describe('The text to reverse')
})

// Create the tools handler
const handle = createTools({
  echo: {
    description: 'Echo back the input',
    inputSchema: z.toJSONSchema(EchoSchema),
    handler: async (input: z.infer<typeof EchoSchema>) => {
      return ToolResponse.text(`Echo: ${input.message}`)
    }
  },
  reverse: {
    description: 'Reverse the input text',
    inputSchema: z.toJSONSchema(ReverseSchema),
    handler: async (input: z.infer<typeof ReverseSchema>) => {
      return ToolResponse.text(input.text.split('').reverse().join(''))
    }
  }
})

// For Spin components
//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})
```

### Manual Implementation

You can also implement the protocol manually with any web framework:

```typescript
import { ToolMetadata, ToolResponse } from 'ftl-sdk';
import { AutoRouter } from 'itty-router';

const router = AutoRouter();

router
  .get('/', async () => {
    // Return array of tool metadata
    const metadata: ToolMetadata[] = [{
      name: 'echo',
      description: 'Echo tool',
      inputSchema: {
        type: 'object',
        properties: {
          message: { type: 'string' }
        },
        required: ['message']
      }
    }, {
      name: 'reverse',
      description: 'Reverse text',
      inputSchema: {
        type: 'object',
        properties: {
          text: { type: 'string' }
        },
        required: ['text']
      }
    }];
    
    return new Response(JSON.stringify(metadata), {
      headers: { 'Content-Type': 'application/json' }
    });
  })
  .post('/:tool', async (request, { params }) => {
    const input = await request.json();
    let response: ToolResponse;
    
    switch (params.tool) {
      case 'echo':
        response = ToolResponse.text(`Echo: ${input.message}`);
        break;
      case 'reverse':
        response = ToolResponse.text(input.text.split('').reverse().join(''));
        break;
      default:
        response = ToolResponse.error(`Tool '${params.tool}' not found`);
    }
    
    return new Response(JSON.stringify(response), {
      headers: { 'Content-Type': 'application/json' }
    });
  });

export default router;
```

## Building to WebAssembly

Tools must be compiled to WebAssembly to run on the Spin platform:

```json
{
  "scripts": {
    "build": "esbuild src/index.ts --bundle --outfile=build/bundle.js --format=esm --platform=browser --external:node:* && j2w -i build/bundle.js -o dist/my-tool.wasm"
  },
  "devDependencies": {
    "@spinframework/build-tools": "^1.0.1"
  }
}
```

The build process:
1. Bundle TypeScript to ESM format using esbuild
2. Convert JavaScript to WebAssembly using `j2w` (js-to-wasm)

## Using with Zod

The SDK integrates with Zod v4's native JSON Schema conversion:

```typescript
import { z } from 'zod'

// Define schema with validation rules
const CalculatorSchema = z.object({
  operation: z.enum(['add', 'subtract', 'multiply', 'divide']),
  a: z.number(),
  b: z.number()
}).refine(
  (data) => data.operation !== 'divide' || data.b !== 0,
  { message: "Cannot divide by zero" }
)

// Convert to JSON Schema - validation rules are preserved
const jsonSchema = z.toJSONSchema(CalculatorSchema)

// Use with createTools
const handle = createTools({
  calculator: {
    description: 'Perform calculations',
    inputSchema: jsonSchema,
    handler: async (input: z.infer<typeof CalculatorSchema>) => {
      // input is fully typed and validated by the gateway
      switch (input.operation) {
        case 'add': return ToolResponse.text(`Result: ${input.a + input.b}`)
        case 'subtract': return ToolResponse.text(`Result: ${input.a - input.b}`)
        case 'multiply': return ToolResponse.text(`Result: ${input.a * input.b}`)
        case 'divide': return ToolResponse.text(`Result: ${input.a / input.b}`)
      }
    }
  }
})
```

## Important: Input Validation

**Tools should NOT validate inputs themselves.** The FTL gateway handles all input validation against your tool's JSON Schema before invoking your handler. This means:

- Your handler can assume all inputs are valid
- Type safety is guaranteed at runtime
- Complex validation rules (like Zod refinements) are enforced by the gateway
- You can focus on business logic, not validation

## API Reference

### `createTools(tools)`

Creates a request handler for multiple MCP tools in a single component.

```typescript
interface ToolDefinition {
  name?: string // Optional explicit name (defaults to property key converted to snake_case)
  description?: string
  inputSchema: JSONSchema
  outputSchema?: JSONSchema
  handler: (input: any) => ToolResponse | Promise<ToolResponse>
}

const handle = createTools({
  [toolName: string]: ToolDefinition
})
```

The returned handler:
- Returns array of tool metadata on GET / requests
- Routes to specific tools on POST /{tool_name} requests
- Automatically converts camelCase property keys to snake_case tool names
- Handles errors gracefully

### `ToolResponse` Helper Methods

```typescript
// Simple text response
ToolResponse.text('Hello, world!')

// Error response
ToolResponse.error('Something went wrong')

// Response with structured content
ToolResponse.withStructured('Operation complete', { result: 42 })
```

### `ToolContent` Helper Methods

```typescript
// Text content
ToolContent.text('Some text', { priority: 0.8 })

// Image content
ToolContent.image(base64Data, 'image/png')

// Audio content
ToolContent.audio(base64Data, 'audio/wav')

// Resource reference
ToolContent.resource({ uri: 'file:///example.txt' })
```

### Type Guards

```typescript
import { isTextContent, isImageContent, isAudioContent, isResourceContent } from 'ftl-sdk'

// Check content types
if (isTextContent(content)) {
  console.log(content.text)
}
```

## Best Practices

1. **Use Zod for Schema Definition**: Leverage Zod's powerful schema capabilities and convert to JSON Schema using `z.toJSONSchema()`.

2. **Trust Input Validation**: Don't validate inputs in your handler - the gateway ensures inputs match your schema.

3. **Keep It Simple**: The SDK is intentionally minimal. Use it for types and basic helpers, not complex abstractions.

4. **Type Safety**: Type your handler parameters directly for full type safety:
   ```typescript
   handler: async (input: z.infer<typeof MySchema>) => {
     // Full type safety for input
     return ToolResponse.text(input.message)
   }
   ```

5. **Error Handling**: Return `ToolResponse.error()` for business logic errors. The SDK handles exceptions automatically.

## Examples

See the [examples directory](https://github.com/fastertools/ftl-cli/tree/main/examples/demo) for complete examples:

- `echo-ts`: Simple echo tool
- `multi-tools-ts`: Multiple tools in one component
- `weather-ts`: External API integration

## License

Apache-2.0