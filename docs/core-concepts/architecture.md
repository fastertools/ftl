# FTL Architecture

FTL's architecture is designed around security, performance, and polyglot composition. This deep dive explains how all the pieces work together to create a robust MCP server platform.

## High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│   MCP Client    │────│  FTL Gateway    │────│ Tool Components │
│  (Claude, etc)  │    │ (mcp-gateway)   │    │     (WASM)      │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                       ┌─────────────────┐
                       │                 │
                       │ FTL Authorizer  │
                       │(mcp-authorizer) │
                       │                 │
                       └─────────────────┘
```

## Core Components

### 1. MCP Gateway

**Purpose**: The central router that provides MCP-compliant access to all tools.

**Responsibilities**:
- **Protocol Implementation**: Full JSON-RPC 2.0 MCP protocol compliance
- **Dynamic Tool Discovery**: Automatically discovers available tools from components
- **Request Routing**: Routes tool calls to appropriate components
- **Schema Validation**: Optional validation of tool arguments
- **Response Assembly**: Formats responses in MCP-compliant JSON-RPC format

**Architecture**:
```
MCP Client Request (JSON-RPC)
         ↓
   ┌─────────────────┐
   │ Protocol Parser │ ← Validates JSON-RPC 2.0 format
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Tool Discovery  │ ← GET http://{component}.spin.internal/
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Request Router  │ ← POST http://{component}.spin.internal/{tool}
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Response Format │ ← Wrap in JSON-RPC response
   └─────────────────┘
         ↓
   MCP Client Response
```

**Key Features**:
- Parallel metadata fetching for performance
- Automatic snake_case to kebab-case tool name conversion
- Comprehensive error handling with standard JSON-RPC error codes
- Optional JSON Schema validation using the `jsonschema` crate

### 2. MCP Authorizer

**Purpose**: High-performance JWT authentication gateway for securing MCP endpoints.

**Authentication Flow**:
```
Client Request (with Bearer token)
         ↓
   ┌─────────────────┐
   │ Token Extract   │ ← Authorization: Bearer {token}
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Token Validate  │ ← JWT signature, issuer, expiry, scopes
   └─────────────────┘
         ↓
   ┌─────────────────┐
   │ Context Headers │ ← x-auth-client-id, x-auth-user-id, etc.
   └─────────────────┘
         ↓
   Forward to MCP Gateway
```

**Security Features**:
- **JWT Validation**: JWKS endpoint or static public key verification
- **OAuth 2.0 Compliance**: Standard-compliant with issuer discovery
- **Scope Enforcement**: Required scope validation per request  
- **WorkOS AuthKit Integration**: Built-in support for WorkOS
- **JWKS Caching**: 5-minute cache reduces provider API calls
- **Token Flexibility**: Support for both JWT and static token validation

### 3. Tool Components (WASM)

**Purpose**: Individual business logic implementations running in secure WebAssembly sandboxes.

**Component Structure**:
```
Tool Component (WASM)
├── Business Logic        ← Your tool implementation
├── Generated Adapter     ← Automatic MCP interface
├── JSON Schema          ← Auto-generated from types
└── Metadata Endpoint    ← Tool discovery information
```

**Capabilities**:
- **Isolated Execution**: Each runs in its own WASM sandbox
- **Language Agnostic**: Rust, Python, Go, TypeScript support
- **Automatic Schema Generation**: JSON schemas from type definitions
- **Outbound Network Access**: Whitelist-controlled HTTP requests
- **Internal Communication**: Access to other components via Spin networking

### 4. Spin Framework Integration

**Purpose**: WebAssembly runtime orchestration and internal networking.

**Runtime Architecture**:
```
┌─────────────────────────────────────────────┐
│               Spin Runtime                  │
├─────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│ │ MCP Gateway │ │ Authorizer  │ │ Tool 1  │ │
│ │   (public)  │ │  (public)   │ │(private)│ │
│ └─────────────┘ └─────────────┘ └─────────┘ │
│         │              │              │     │
│ ┌─────────────────────────────────────────┐ │
│ │      *.spin.internal Networking       │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
         │
┌─────────────────┐
│  HTTP Triggers  │ ← External access points
└─────────────────┘
```

**Key Features**:
- **Internal Networking**: `http://*.spin.internal` domains for component communication
- **HTTP Triggers**: Public endpoint routing to appropriate components
- **Configuration Management**: `ftl.toml` → `spin.toml` transpilation
- **Component Lifecycle**: Automatic loading, health monitoring, and sandboxing

## Request Flow Deep Dive

Let's trace a complete request from an MCP client to a tool:

### 1. Client Request
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "text-analyzer/analyze",
    "arguments": {
      "content": "Hello, world!",
      "options": { "sentiment": true }
    }
  }
}
```

### 2. Authentication (if enabled)
```
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGci...
                ↓
MCP Authorizer validates JWT:
- Signature verification via JWKS
- Issuer, audience, expiration checks  
- Required scope validation
                ↓
Headers added:
x-auth-client-id: client-123
x-auth-user-id: user-456
x-auth-issuer: https://auth.example.com
```

### 3. Gateway Processing
```
JSON-RPC Request → Protocol Validation → Tool Discovery
                                              ↓
GET http://text-analyzer.spin.internal/ 
    ← Returns tool metadata and schemas
                                              ↓
Argument Validation (optional) → Request Routing
```

### 4. Tool Invocation
```
POST http://text-analyzer.spin.internal/analyze
Content-Type: application/json

{
  "content": "Hello, world!",
  "options": { "sentiment": true }
}
```

### 5. Component Execution
```
WASM Component (text-analyzer)
         ↓
Business Logic Execution
         ↓
{
  "sentiment_score": 0.8,
  "confidence": 0.95,
  "language": "en"
}
```

### 6. Response Assembly
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"sentiment_score\": 0.8, \"confidence\": 0.95, \"language\": \"en\"}"
      }
    ]
  }
}
```

## Configuration Architecture

### FTL Project Configuration (`ftl.toml`)
```toml
[project]
name = "my-project"
access_control = "private"  # Enables authentication

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = true

[oidc]  # Custom OAuth provider
issuer = "https://auth.example.com"
audience = "api-identifier"

[variables]
custom_api_key = { default = "demo-key" }

[tools.text-analyzer]
path = "components/text-analyzer"
allowed_outbound_hosts = ["https://api.openai.com"]
```

### Generated Spin Configuration (`spin.toml`)
```toml
spin_manifest_version = "2"
name = "my-project"

[variables]
tool_components = { default = "text-analyzer" }
oidc_issuer = { default = "https://auth.example.com" }

[[component]]
id = "mcp-gateway"
source = { url = "ghcr.io/fastertools/mcp-gateway:0.0.11" }

[component.trigger]
route = "/..."
```

**Transpilation Process**:
1. **Tool Discovery**: Scan `[tools.*]` sections in `ftl.toml`
2. **Component Generation**: Create Spin component definitions
3. **Variable Injection**: Template variables into component configs
4. **Route Configuration**: Set up HTTP triggers and routing rules
5. **Security Integration**: Configure auth components based on access_control

## Security Model

### WebAssembly Sandboxing
```
┌─────────────────────────────────────────┐
│              Host System                │
├─────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────┐ │
│ │   Tool A    │ │   Tool B    │ │ ... │ │
│ │   ┌─────┐   │ │   ┌─────┐   │ │     │ │
│ │   │WASM │   │ │   │WASM │   │ │     │ │
│ │   │Heap │   │ │   │Heap │   │ │     │ │
│ │   └─────┘   │ │   └─────┘   │ │     │ │
│ └─────────────┘ └─────────────┘ └─────┘ │
│                                         │
│ ┌─────────────────────────────────────┐ │
│ │       Wasmtime Runtime              │ │
│ └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

**Isolation Guarantees**:
- **Memory Isolation**: Each component has separate linear memory space
- **Capability-Based Security**: Explicit permission grants for system access
- **Network Restrictions**: Outbound requests limited to whitelisted hosts
- **Crash Isolation**: Component failures don't affect other components
- **Resource Limits**: CPU and memory usage controlled by runtime

### Network Security
```
Internet         Internal Network     Components
   │                    │                 │
   ├─ /tools/call ──────┼─ mcp-gateway ───┼─ Tool Components
   │                    │      │          │
   ├─ /auth/* ──────────┼─ authorizer     │
   │                    │                 │
   └─ (blocked) ────────┼─ *.spin.internal (private)
```

**Security Layers**:
- **Public Endpoints**: Only gateway and authorizer accessible externally
- **Private Network**: Tool components only accessible via internal network
- **TLS Termination**: HTTPS support with optional certificate management
- **CORS Support**: Cross-origin request handling for web clients

## Performance Characteristics

### Component Communication
- **In-Process**: WASM components run within the same process
- **Memory Sharing**: Efficient data transfer via shared memory regions
- **Connection Pooling**: Persistent HTTP connections between components
- **Parallel Processing**: Concurrent tool execution and discovery

### Scalability Patterns
```
Single Instance (Development)
┌─────────────────┐
│ FTL Application │
│   All Components│
└─────────────────┘

Horizontal Scaling (Production)
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ FTL Instance 1  │ │ FTL Instance 2  │ │ FTL Instance N  │
│   All Tools     │ │   All Tools     │ │   All Tools     │
└─────────────────┘ └─────────────────┘ └─────────────────┘
         │                   │                   │
    ┌─────────────────────────────────────────────────┐
    │              Load Balancer                      │
    └─────────────────────────────────────────────────┘
```

## Deployment Architecture

### Local Development
```bash
ftl up  # Single process, all components
```

### Production Deployment  
```bash
ftl eng deploy  # Managed deployment to FTL Engine
```

### Container Deployment
```dockerfile
FROM ghcr.io/fermyon/spin:canary
COPY spin.toml ftl.toml ./
COPY components/ components/
```

## Extension Points

### Custom Components
- Implement MCP protocol in any language that compiles to WASM
- Use Component Model interfaces for language interoperability
- Register components in `ftl.toml` configuration

### Authentication Providers
- Custom JWT issuers via OIDC configuration
- Static token validation for development
- WorkOS AuthKit integration for production

### Middleware Components
- Request/response transformation
- Logging and monitoring integration
- Custom protocol adaptations

## Next Steps

- **Implementation Details**: Learn the development workflow in [Project Lifecycle](./lifecycle.md)
- **Practical Application**: Try building with [Getting Started Tutorials](../getting-started/)
- **Advanced Patterns**: Explore [How-to Guides](../guides/) and [Examples](../../examples/)

FTL's architecture provides a secure, performant foundation for polyglot AI tool composition, with WebAssembly providing isolation and Spin providing orchestration.