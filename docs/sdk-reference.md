# SDK Reference

This document provides a reference for using the wasmcp SDKs with FTL. For complete SDK documentation, see the [wasmcp repository](https://github.com/fastertools/wasmcp).

## TypeScript/JavaScript SDK

### Installation

The wasmcp SDK is automatically included when you create a new component with `ftl add`. The templates handle the dependency management for you.

### Core Functions

#### `createHandler(features)`

Creates an MCP handler with the specified features.

```typescript
import { createHandler } from 'wasmcp';

export const handler = createHandler({
  tools: [...],      // Array of tools
  resources: [...],  // Array of resources
  prompts: [...]     // Array of prompts
});
```

#### `createTool(config)`

Creates a tool that can be called by AI agents.

```typescript
const tool = createTool({
  name: string,              // Unique tool identifier
  description: string,       // Human-readable description
  inputSchema: object,       // JSON Schema for input validation
  execute: async (args) => string  // Tool implementation
});
```

**Example:**
```typescript
const calculateTool = createTool({
  name: 'calculate',
  description: 'Perform mathematical calculations',
  inputSchema: {
    type: 'object',
    properties: {
      expression: { 
        type: 'string',
        description: 'Mathematical expression to evaluate'
      }
    },
    required: ['expression']
  },
  execute: async (args) => {
    const result = evaluateExpression(args.expression);
    return `Result: ${result}`;
  }
});
```

#### `createResource(config)`

Creates a resource that can be read by AI agents.

```typescript
const resource = createResource({
  uri: string,               // Resource URI
  name: string,              // Display name
  description?: string,      // Resource description
  mimeType?: string,         // Content MIME type
  read: async () => string   // Resource reader
});
```

**Example:**
```typescript
const configResource = createResource({
  uri: 'config://app-settings',
  name: 'Application Settings',
  description: 'Current application configuration',
  mimeType: 'application/json',
  read: async () => {
    const config = await loadConfig();
    return JSON.stringify(config, null, 2);
  }
});
```

#### `createPrompt(config)`

Creates a reusable prompt template.

```typescript
const prompt = createPrompt({
  name: string,              // Prompt identifier
  description?: string,      // Prompt description
  arguments?: Array<{        // Prompt arguments
    name: string,
    description?: string,
    required?: boolean
  }>,
  resolve: async (args) => PromptMessage[]
});

interface PromptMessage {
  role: 'user' | 'assistant';
  content: string;
}
```

**Example:**
```typescript
const analysisPrompt = createPrompt({
  name: 'analyze_code',
  description: 'Generate code analysis prompt',
  arguments: [
    { name: 'language', description: 'Programming language', required: true },
    { name: 'code', description: 'Code to analyze', required: true }
  ],
  resolve: async (args) => {
    return [
      {
        role: 'user',
        content: `Analyze this ${args.language} code:\n\n${args.code}`
      },
      {
        role: 'assistant',
        content: 'I\'ll analyze this code for bugs, performance, and best practices.'
      }
    ];
  }
});
```

### Types

```typescript
interface Tool<TArgs = any> {
  name: string;
  description: string;
  inputSchema: object;
  execute: (args: TArgs) => string | Promise<string>;
}

interface Resource {
  uri: string;
  name: string;
  description?: string;
  mimeType?: string;
  read: () => string | Promise<string>;
}

interface Prompt<TArgs = any> {
  name: string;
  description?: string;
  arguments?: Array<{
    name: string;
    description?: string;
    required?: boolean;
  }>;
  resolve: (args: TArgs) => PromptMessage[] | Promise<PromptMessage[]>;
}
```

## Rust SDK

### Installation

The wasmcp SDK is automatically included when you create a new Rust component with `ftl add`. The templates handle the dependency management for you.

### Core Macros

#### `create_handler!`

Creates an MCP handler with the specified functions.

```rust
use wasmcp::*;

create_handler!(
    tools: get_tools,
    resources: get_resources,
    prompts: get_prompts
);

fn get_tools() -> Vec<Tool> { vec![] }
fn get_resources() -> Vec<Resource> { vec![] }
fn get_prompts() -> Vec<Prompt> { vec![] }
```

#### `tool!`

Creates a tool definition.

```rust
tool!(
    name: &str,              // Tool name
    description: &str,       // Tool description
    schema: serde_json::Value,  // Input schema
    handler: fn              // Handler function
)
```

**Example:**
```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct CalculateArgs {
    expression: String,
}

fn calculate(args: CalculateArgs) -> Result<String, String> {
    let result = evaluate_expression(&args.expression)?;
    Ok(format!("Result: {}", result))
}

fn get_tools() -> Vec<Tool> {
    vec![
        tool!(
            "calculate",
            "Perform mathematical calculations",
            json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression"
                    }
                },
                "required": ["expression"]
            }),
            calculate
        )
    ]
}
```

#### `resource!`

Creates a resource definition.

```rust
resource!(
    uri: &str,               // Resource URI
    name: &str,              // Display name
    mime_type: &str,         // MIME type
    reader: fn() -> Result<String, String>
)
```

**Example:**
```rust
fn read_config() -> Result<String, String> {
    let config = load_configuration()?;
    serde_json::to_string_pretty(&config)
        .map_err(|e| e.to_string())
}

fn get_resources() -> Vec<Resource> {
    vec![
        resource!(
            "config://settings",
            "Application Settings",
            "application/json",
            read_config
        )
    ]
}
```

#### `prompt!`

Creates a prompt definition.

```rust
prompt!(
    name: &str,              // Prompt name
    description: &str,       // Description
    arguments: Vec<PromptArg>,  // Arguments
    resolver: fn             // Resolver function
)
```

**Example:**
```rust
#[derive(Deserialize)]
struct AnalyzeArgs {
    language: String,
    code: String,
}

fn analyze_prompt(args: AnalyzeArgs) -> Result<Vec<PromptMessage>, String> {
    Ok(vec![
        PromptMessage {
            role: Role::User,
            content: format!("Analyze this {} code:\n\n{}", 
                           args.language, args.code),
        },
        PromptMessage {
            role: Role::Assistant,
            content: "I'll analyze this code for bugs and improvements.".to_string(),
        }
    ])
}

fn get_prompts() -> Vec<Prompt> {
    vec![
        prompt!(
            "analyze_code",
            "Generate code analysis prompt",
            vec![
                PromptArg::new("language", "Programming language", true),
                PromptArg::new("code", "Code to analyze", true),
            ],
            analyze_prompt
        )
    ]
}
```

### Types

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub handler: Box<dyn Fn(serde_json::Value) -> Result<String, String>>,
}

pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub reader: Box<dyn Fn() -> Result<String, String>>,
}

pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArg>,
    pub resolver: Box<dyn Fn(serde_json::Value) -> Result<Vec<PromptMessage>, String>>,
}

pub struct PromptArg {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

pub struct PromptMessage {
    pub role: Role,
    pub content: String,
}

pub enum Role {
    User,
    Assistant,
}
```

## Error Handling

### TypeScript/JavaScript

Return error information as part of the response:

```typescript
execute: async (args) => {
  try {
    const result = await riskyOperation(args);
    return JSON.stringify({ success: true, data: result });
  } catch (error) {
    return JSON.stringify({ 
      success: false, 
      error: error.message,
      code: error.code || 'UNKNOWN_ERROR'
    });
  }
}
```

### Rust

Use `Result<String, String>` for error handling:

```rust
fn my_tool(args: MyArgs) -> Result<String, String> {
    match risky_operation(&args) {
        Ok(result) => Ok(format!("Success: {}", result)),
        Err(e) => Err(format!("Error: {}", e)),
    }
}
```

## Best Practices

### 1. Input Validation

Always validate inputs using JSON Schema:

```typescript
inputSchema: {
  type: 'object',
  properties: {
    url: { 
      type: 'string', 
      format: 'uri',
      pattern: '^https?://'
    },
    timeout: {
      type: 'integer',
      minimum: 1,
      maximum: 30000
    }
  },
  required: ['url'],
  additionalProperties: false
}
```

### 2. Async Operations

Handle async operations properly:

```typescript
// TypeScript
execute: async (args) => {
  const results = await Promise.all([
    fetchData(args.url1),
    fetchData(args.url2)
  ]);
  return JSON.stringify(results);
}
```

```rust
// Rust
use tokio::runtime::Runtime;

fn async_tool(args: Args) -> Result<String, String> {
    let rt = Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        let data = fetch_data(&args.url).await?;
        Ok(format!("Data: {}", data))
    })
}
```

### 3. Resource Cleanup

Always clean up resources:

```typescript
let client;
try {
  client = await createClient(args.config);
  return await client.query(args.query);
} finally {
  if (client) {
    await client.close();
  }
}
```

### 4. Structured Responses

Return structured data when possible:

```typescript
execute: async (args) => {
  const result = {
    status: 'success',
    data: {
      count: 42,
      items: ['a', 'b', 'c']
    },
    metadata: {
      timestamp: new Date().toISOString(),
      version: '1.0.0'
    }
  };
  return JSON.stringify(result, null, 2);
}
```

## Migration Guide

### From MCP SDK to FTL SDK

If you're migrating from the standard MCP SDK:

**Before (MCP SDK):**
```typescript
const server = new Server({
  name: 'my-server',
  version: '1.0.0'
});

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return { tools: [...] };
});
```

**After (FTL SDK):**
```typescript
export const handler = createHandler({
  tools: [...],
  resources: [...],
  prompts: [...]
});
```

The FTL SDK handles all the MCP protocol details for you!