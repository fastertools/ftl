# Component Development

This guide covers everything you need to know about developing MCP components with FTL.

## Component Structure

Every FTL component follows a consistent structure:

<pre>
my-component/
├── ftl.toml            # Component metadata
├── Makefile            # Build automation
├── handler/            # Component source code
│   ├── [package.json | Cargo.toml]  # Language-specific manifest
│   ├── src/            # Source files
│   └── test/           # Test files
└── [.wit/]             # WebAssembly Interface Types (generated)
</pre>

### ftl.toml

The component metadata file:

```toml
name = "my-component"
version = "0.1.0"
description = "My awesome MCP component"
route = "/my-component"
```

### Makefile

Standard targets for all components:
- `make build` - Build the component
- `make test` - Run tests
- `make clean` - Clean build artifacts

## Language-Specific Development

### TypeScript Components

#### Project Setup

```bash
ftl add my-tool --language typescript
cd my-tool/handler
```

#### Implementation

```typescript
// src/features.ts
import { createTool, createResource, createPrompt } from 'ftl-mcp';

// Define tools
export const tools = [
  createTool({
    name: 'get_weather',
    description: 'Get weather for a location',
    inputSchema: {
      type: 'object',
      properties: {
        location: { type: 'string', description: 'City name' },
        units: { type: 'string', enum: ['celsius', 'fahrenheit'] }
      },
      required: ['location']
    },
    execute: async (args) => {
      // Implementation
      return `Weather in ${args.location}: 72°F`;
    }
  })
];

// Define resources
export const resources = [
  createResource({
    uri: 'weather://current',
    name: 'Current Weather Data',
    description: 'Real-time weather information',
    mimeType: 'application/json',
    read: async () => {
      return JSON.stringify({ temp: 72, conditions: 'sunny' });
    }
  })
];

// Define prompts
export const prompts = [
  createPrompt({
    name: 'weather_report',
    description: 'Generate a weather report',
    arguments: [
      { name: 'location', description: 'Location for weather', required: true }
    ],
    resolve: async (args) => {
      return [
        { role: 'user', content: `What's the weather in ${args.location}?` },
        { role: 'assistant', content: `I'll check the weather for ${args.location}.` }
      ];
    }
  })
];
```

#### Testing

```typescript
// test/weather.test.ts
import { describe, it, expect } from 'vitest';
import { tools } from '../src/features';

describe('Weather Tool', () => {
  it('should return weather data', async () => {
    const weatherTool = tools.find(t => t.name === 'get_weather');
    const result = await weatherTool?.execute({ 
      location: 'San Francisco' 
    });
    expect(result).toContain('San Francisco');
  });
});
```

### Rust Components

#### Project Setup

```bash
ftl add my-tool --language rust
cd my-tool/handler
```

#### Implementation

```rust
// src/lib.rs
use ftl-mcp::*;
use serde::{Deserialize, Serialize};

// Define handler
create_handler!(
    tools: get_tools,
    resources: get_resources,
    prompts: get_prompts
);

// Tool implementation
#[derive(Deserialize)]
struct WeatherArgs {
    location: String,
    units: Option<String>,
}

fn get_weather(args: WeatherArgs) -> Result<String, String> {
    Ok(format!("Weather in {}: 72°F", args.location))
}

// Export tools
fn get_tools() -> Vec<Tool> {
    vec![
        tool!(
            "get_weather",
            "Get weather for a location",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" },
                    "units": { "type": "string" }
                },
                "required": ["location"]
            }),
            get_weather
        )
    ]
}

// Export resources
fn get_resources() -> Vec<Resource> {
    vec![
        resource!(
            "weather://current",
            "Current Weather Data",
            "application/json",
            || Ok(r#"{"temp": 72, "conditions": "sunny"}"#.to_string())
        )
    ]
}

// Export prompts
fn get_prompts() -> Vec<Prompt> {
    vec![]
}
```

#### Testing

```rust
// src/lib.rs (test module)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_tool() {
        let args = WeatherArgs {
            location: "San Francisco".to_string(),
            units: None,
        };
        let result = get_weather(args).unwrap();
        assert!(result.contains("San Francisco"));
    }
}
```

### JavaScript Components

JavaScript components follow the same pattern as TypeScript but without type annotations:

```javascript
// src/features.js
import { createTool } from 'ftl-mcp';

export const tools = [
  createTool({
    name: 'calculate',
    description: 'Perform calculations',
    inputSchema: {
      type: 'object',
      properties: {
        expression: { type: 'string' }
      },
      required: ['expression']
    },
    execute: async (args) => {
      // Simple example - in production use a safe parser
      try {
        const result = eval(args.expression);
        return `Result: ${result}`;
      } catch (error) {
        return `Error: Invalid expression`;
      }
    }
  })
];
```

## Component Dependencies

### Rust Components

Add dependencies to `handler/Cargo.toml`:

```toml
[dependencies]
ftl-mcp = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

Note: cargo-component is automatically installed when building Rust components.

### TypeScript/JavaScript Components

Add dependencies to `handler/package.json`:

```json
{
  "dependencies": {
    "ftl-mcp": "^0.1.0",
    "axios": "^1.6.0",
    "dotenv": "^16.0.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0",
    "vitest": "^1.0.0"
  }
}
```

## Build Process

### Development Builds

```bash
# Build a single component
cd my-component
make build

# Build all components (from project root)
ftl build
```

### Production Builds

```bash
# Build with optimizations
ftl build --release
```

### Automatic Rebuilds

```bash
# Watch for changes and rebuild
ftl watch
```

## Testing

### Unit Tests

Each language has its own test runner:

```bash
# Run tests for a component
cd my-component
make test

# Run all tests (from project root)
ftl test
```

### Integration Testing

Test your component with MCP clients:

```javascript
// test-client.js
import { Client } from '@modelcontextprotocol/sdk';

const client = new Client({
  url: 'http://localhost:3000/my-component/mcp'
});

// List tools
const tools = await client.listTools();
console.log(tools);

// Call a tool
const result = await client.callTool('my_tool', {
  input: 'test data'
});
console.log(result);
```

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```typescript
execute: async (args) => {
  try {
    const result = await someOperation(args);
    return JSON.stringify(result);
  } catch (error) {
    return JSON.stringify({ 
      error: error.message,
      code: 'OPERATION_FAILED'
    });
  }
}
```

### 2. Input Validation

Use JSON Schema for comprehensive validation:

```typescript
inputSchema: {
  type: 'object',
  properties: {
    email: { 
      type: 'string', 
      format: 'email',
      description: 'User email address'
    },
    age: { 
      type: 'integer',
      minimum: 0,
      maximum: 150
    }
  },
  required: ['email']
}
```

### 3. Async Operations

Handle async operations properly:

```rust
async fn fetch_data(url: String) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    Ok(body)
}
```

### 4. Resource Management

Clean up resources properly:

```typescript
let connection;
try {
  connection = await createConnection();
  return await connection.query(args.query);
} finally {
  if (connection) {
    await connection.close();
  }
}
```

### 5. Documentation

Document your tools thoroughly:

```typescript
createTool({
  name: 'analyze_data',
  description: 'Analyze data using various statistical methods. ' +
               'Supports CSV, JSON, and Excel formats. ' +
               'Returns summary statistics and visualizations.',
  // ...
})
```

## Advanced Topics

### WebAssembly Interface Types

FTL generates WIT files for language interop:

```wit
// Generated .wit/mcp.wit
interface mcp-handler {
  record tool {
    name: string,
    description: string,
    input-schema: string,
  }
  
  list-tools: func() -> list<tool>
  call-tool: func(name: string, args: string) -> result<string, error>
}
```

### Component Composition

Combine multiple components in `spin.toml`:

```toml
[[component]]
id = "weather"
route = "/weather/..."
source = "weather-tool/handler/target/wasm32-wasip1/release/handler.wasm"

[[component]]
id = "news"  
route = "/news/..."
source = "news-tool/handler/dist/handler.wasm"
```

### Performance Optimization

1. **Minimize dependencies**: Only include what you need
2. **Use streaming**: For large responses, consider streaming
3. **Cache results**: Implement caching for expensive operations
4. **Profile your code**: Use language-specific profiling tools

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
make clean
make build

# Check for missing dependencies
npm install  # for JS/TS
cargo check  # for Rust
```

### Runtime Errors

Check the Spin logs:
```bash
ftl up --follow
```

### Test Failures

Run tests with verbose output:
```bash
npm test -- --reporter=verbose  # JS/TS
cargo test -- --nocapture       # Rust
```

## Next Steps

- [Publishing Components](./publishing.md) - Share your components
- [SDK Reference](./sdk-reference.md) - Detailed API documentation
- [Deployment Guide](./deployment.md) - Deploy to production