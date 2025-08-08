# SDK Reference

This section provides comprehensive API documentation for each FTL SDK. Choose your language to access detailed reference materials, code examples, and best practices.

## Available SDKs

### [Rust SDK](./rust/)
**Status**: Primary SDK ✅  
**Language**: Rust 1.70+  
**Features**: Full feature support, procedural macros, async/await  
**Best For**: High-performance tools, system integration, complex business logic

### [Python SDK](./python/)
**Status**: Stable ✅  
**Language**: Python 3.10+  
**Features**: Type hints, async support, pydantic integration  
**Best For**: Data science, AI/ML, rapid prototyping, scripting

### [Go SDK](./go/)
**Status**: Stable ✅  
**Language**: Go 1.21+ with TinyGo  
**Features**: Concurrent tools, structured types, performance  
**Best For**: Network services, concurrent processing, system tools

### [TypeScript SDK](./typescript/)
**Status**: Beta ⚡  
**Language**: TypeScript/JavaScript (Node.js)  
**Features**: Type safety, modern async/await, JSON handling  
**Best For**: Web APIs, JSON processing, integration tools

## SDK Architecture

All FTL SDKs follow a consistent architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    Your Tool Code                       │
│  ┌─────────────────┐ ┌─────────────────┐ ┌──────────┐  │
│  │   @tool         │ │   @tool         │ │   ...    │  │
│  │   function      │ │   function      │ │          │  │
│  └─────────────────┘ └─────────────────┘ └──────────┘  │
├─────────────────────────────────────────────────────────┤
│                  Language SDK                           │
│  ┌─────────────────┐ ┌─────────────────┐ ┌──────────┐  │
│  │ Tool Decorator  │ │ ToolResponse    │ │ Schemas  │  │
│  │ & Registration  │ │ Types           │ │ Gen      │  │
│  └─────────────────┘ └─────────────────┘ └──────────┘  │
├─────────────────────────────────────────────────────────┤
│               WebAssembly Component Model               │
│  ┌─────────────────┐ ┌─────────────────┐ ┌──────────┐  │
│  │ Generated       │ │ Type Adapters   │ │ WASM     │  │
│  │ Bindings        │ │ & Marshaling    │ │ Runtime  │  │
│  └─────────────────┘ └─────────────────┘ └──────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Common Patterns Across SDKs

### Tool Definition
Every SDK provides a decorator/macro to define tools:

```rust
// Rust
#[tool]
pub fn my_tool(input: String) -> ToolResponse { /* ... */ }
```

```python
# Python
@tool
def my_tool(input: str) -> ToolResponse: ...
```

```go
// Go
func (t *ToolImpl) MyTool(ctx context.Context, input string) cm.Result[string, string] { /* ... */ }
```

```typescript
// TypeScript
@tool
export function myTool(input: string): ToolResponse { /* ... */ }
```

### Response Types
All SDKs provide structured response types:

- **Success responses**: Return data with optional metadata
- **Error responses**: Return error messages with context
- **Type safety**: Compile-time validation of response formats

### Schema Generation
Automatic JSON Schema generation from type definitions:

- **Input validation**: Ensures tools receive correctly typed data
- **MCP compliance**: Generates standard MCP tool descriptions
- **Documentation**: Self-documenting APIs from type information

## Using This Reference

### For New Users
1. Start with your preferred language's overview page
2. Follow the quick start guide
3. Review common patterns and examples
4. Reference specific APIs as needed

### For Experienced Developers
1. Jump to specific API sections
2. Check advanced patterns and best practices
3. Review migration guides for updates
4. Explore cross-language integration patterns

### Navigation Tips
- **Search**: Use your browser's find function to locate specific APIs
- **Examples**: Look for 📝 Example blocks throughout the documentation
- **Links**: Cross-references help you find related functionality
- **Code Snippets**: All examples are tested and ready to use

## API Documentation Standards

Each SDK reference includes:

### Function Signatures
Complete type information with parameter and return types:

```rust
pub fn tool_name(
    param1: Type1,
    param2: Option<Type2>
) -> Result<ReturnType, Error>
```

### Parameter Descriptions
- **Type**: Parameter type and constraints
- **Required**: Whether the parameter is optional
- **Default**: Default values where applicable
- **Validation**: Input validation rules

### Code Examples
Working code snippets for common use cases:

📝 **Example**: Basic tool implementation
```rust
#[tool]
pub fn hello_world(name: Option<String>) -> ToolResponse {
    let name = name.unwrap_or_else(|| "World".to_string());
    ToolResponse::ok(&format!("Hello, {}!", name))
}
```

### Error Conditions
Common error cases and how to handle them:

- **Invalid input**: Malformed or missing parameters
- **Runtime errors**: Network failures, file access issues
- **Type errors**: Schema validation failures

## Version Compatibility

| SDK Version | FTL CLI Version | Status | Notes |
|-------------|----------------|--------|--------|
| 0.1.x | 0.0.40+ | Current | Stable API |
| 0.2.x | 0.1.0+ | Planned | Enhanced async support |

## Migration Guides

When SDKs are updated, migration guides help you upgrade:

- **Breaking changes**: API changes that require code updates
- **New features**: Enhanced functionality and capabilities
- **Deprecations**: Features being phased out
- **Examples**: Before/after code comparisons

## Getting Help

### For API Questions
- Check the specific SDK documentation
- Look for similar patterns in examples
- Review the Core Concepts for architectural understanding

### For Bugs or Issues
- Check existing issues in the [GitHub repository](https://github.com/fastertools/ftl)
- Create a new issue with code examples and error messages
- Include your SDK version and environment details

### For Feature Requests
- Propose new APIs or functionality
- Provide use cases and examples
- Discuss with the community in GitHub Discussions

## Contributing to SDK Documentation

We welcome contributions to improve the SDK documentation:

1. **Corrections**: Fix typos, clarify explanations
2. **Examples**: Add more code examples and use cases  
3. **Coverage**: Document missing APIs or edge cases
4. **Improvements**: Better organization or clearer explanations

See [Contributing](../contributing/) for guidelines on submitting documentation improvements.

---

Choose your language below to dive into the detailed SDK reference: