# {{project-name}}

An FTL MCP tool written in TypeScript.

## Prerequisites

- Node.js 18 or higher
- npm or yarn package manager

## Quick Start

1. **Install dependencies:**
   ```bash
   ftl build
   # This runs `make build` which handles npm install and compilation
   
   # Or use make/npm directly:
   make install    # npm install
   make build      # Build WebAssembly module
   ```

2. **Run the MCP server:**
   ```bash
   ftl up
   ```

## Development

### Project Structure

```
{{project-name}}/
├── src/
│   └── index.ts         # Tool implementation
├── package.json         # Project configuration and dependencies
├── tsconfig.json        # TypeScript configuration
├── Makefile             # Development tasks and build automation
└── README.md
```

### Available Commands

```bash
make build       # Build WebAssembly module (includes typecheck)
make clean       # Clean build artifacts and node_modules
make test        # Run tests
make format      # Format code with prettier
make lint        # Run ESLint
make install     # Install npm dependencies
make dev         # Run install, format, lint, test (full development check)
```

### Adding New Tools

Edit `src/index.ts` to add new tools using the `createTools` API with Zod schemas:

```typescript
import { createTools, ToolResponse } from 'ftl-sdk'
import * as z from 'zod'

// Define input schemas using Zod
const CalculatorSchema = z.object({
  a: z.number().describe('First number'),
  b: z.number().describe('Second number')
})

const TextProcessorSchema = z.object({
  text: z.string().describe('Text to process'),
  operation: z.enum(['uppercase', 'lowercase', 'reverse']).describe('Operation to perform')
})

const handle = createTools({
  // Calculator tool
  add: {
    description: 'Adds two numbers together',
    inputSchema: z.toJSONSchema(CalculatorSchema),
    handler: async (input: z.infer<typeof CalculatorSchema>) => {
      const result = input.a + input.b
      return ToolResponse.json({ result })
    }
  },
  
  // Text processing tool
  processText: {
    description: 'Processes text with various operations',
    inputSchema: z.toJSONSchema(TextProcessorSchema),
    handler: async (input: z.infer<typeof TextProcessorSchema>) => {
      let result: string
      
      switch (input.operation) {
        case 'uppercase':
          result = input.text.toUpperCase()
          break
        case 'lowercase':
          result = input.text.toLowerCase()
          break
        case 'reverse':
          result = input.text.split('').reverse().join('')
          break
      }
      
      return ToolResponse.text(`Result: ${result}`)
    }
  }
})

//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})
```

**Key Points:**
- Use **Zod** for schema definition and validation
- Call `z.toJSONSchema()` to convert Zod schemas to MCP tool schemas
- Use `z.infer<typeof Schema>` for TypeScript type inference
- Use `ToolResponse.text()` for simple text or `ToolResponse.json()` for structured responses
- The SDK automatically handles MCP protocol and validation

### Testing

Add test scripts to `package.json`:

```json
{
  "scripts": {
    "test": "jest",
    "test:watch": "jest --watch",
    "format": "prettier --write src",
    "lint": "eslint src --ext .ts,.tsx"
  },
  "devDependencies": {
    "esbuild": "^0.19.0",
    "typescript": "^5.8.3",
    "jest": "^29.0.0",
    "@types/jest": "^29.0.0",
    "eslint": "^8.0.0",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "prettier": "^3.0.0"
  }
}
```

Create tests in `src/__tests__/` or alongside your code:

```typescript
// src/__tests__/index.test.ts
import { describe, expect, test } from '@jest/globals'

describe('Calculator Tool', () => {
  test('adds two numbers', () => {
    const result = 2 + 3
    expect(result).toBe(5)
  })
})
```

Run tests with:
```bash
npm test
# Or: make test
```

### Code Quality

Add linting and formatting to `package.json`:

```json
{
  "scripts": {
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "format": "prettier --write src",
    "format:check": "prettier --check src"
  },
  "devDependencies": {
    "eslint": "^8.0.0",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "prettier": "^3.0.0"
  }
}
```

Run quality checks:
```bash
make dev
# This runs: install, format, lint, and test
```

### WebAssembly Build Process

The build process:
1. **TypeScript compilation**: `tsc --noEmit` (type checking only)
2. **Bundling**: `esbuild` bundles your TypeScript code
3. **WASM generation**: `j2w` (JavaScript-to-WASM) converts the bundle to WebAssembly

Build configuration in `package.json`:
```json
{
  "scripts": {
    "build": "npm run typecheck && esbuild src/index.ts --bundle --outfile=build/bundle.js --format=esm --platform=browser --external:node:* && mkdir -p dist && j2w -i build/bundle.js -o dist/{{project-name | kebab_case}}.wasm"
  }
}
```

## Running Your Tool

After building, start the local development server:

```bash
ftl up
```

Your MCP server will be available at `http://localhost:3000/` and can be used with any MCP-compatible client.

## Troubleshooting

**Build fails with "j2w not found":**
The `j2w` (JavaScript-to-WASM) tool is included in `@spinframework/build-tools` dependency. Make sure you've run `npm install` to get the build tools.

**TypeScript compilation errors:**
Check `tsconfig.json` settings and ensure all dependencies have proper type definitions:
```bash
npm install @types/node --save-dev
```

**WebAssembly size issues:**
- Use tree-shaking friendly imports: `import { specific } from 'library'`
- Avoid large dependencies that don't tree-shake well
- Consider using lighter alternatives to heavy libraries

**Fetch event not working:**
Ensure you have the fetch event listener at the end of your `index.ts`:
```typescript
//@ts-ignore
addEventListener('fetch', (event: FetchEvent) => {
  event.respondWith(handle(event.request))
})
```

## Advanced Usage

### Async Operations

All tool handlers are async by default:

```typescript
asyncTool: {
  description: 'Performs async operations',
  inputSchema: z.toJSONSchema(SomeSchema),
  handler: async (input: z.infer<typeof SomeSchema>) => {
    // Fetch from external API
    const response = await fetch('https://api.example.com/data')
    const data = await response.json()
    
    return ToolResponse.json(data)
  }
}
```

### Error Handling

Use ToolResponse.error() for proper error responses:

```typescript
handler: async (input: z.infer<typeof Schema>) => {
  try {
    // Tool logic here
    return ToolResponse.text('Success')
  } catch (error) {
    return ToolResponse.error(`Failed: ${error.message}`)
  }
}
```