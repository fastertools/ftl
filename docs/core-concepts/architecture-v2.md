# FTL Architecture

FTL's architecture is designed around security, performance, and polyglot composition. This deep dive explains how all the pieces work together to create a robust MCP server platform with WebAssembly at its core.

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [High-Level Architecture](#high-level-architecture)
3. [Core Components](#core-components)
4. [Configuration System](#configuration-system)
5. [SDK Architecture](#sdk-architecture)
6. [Request Flow Deep Dive](#request-flow-deep-dive)

## Executive Summary

FTL represents a sophisticated, WebAssembly-based framework for building and deploying polyglot Model Context Protocol (MCP) servers. The architecture demonstrates several key innovations:

- **Dynamic Configuration Transpilation**: Simple `ftl.toml` configs expand into complex `spin.toml` runtime configurations
- **Component-Based Security**: WebAssembly sandboxing with OAuth 2.0/JWT authentication
- **Polyglot Tool Composition**: Multiple languages (Rust, Python, Go, TypeScript) unified through WASM Component Model  
- **Edge-Optimized Deployment**: Sub-millisecond cold starts with global distribution capabilities

**Observed Production Performance**:
- Global Average Response: 33ms
- Global P95 Response: 131ms
- Cold Start Time: Sub-millisecond

## High-Level Architecture

```mermaid
graph TB
    Client[MCP Client<br/>Claude, etc] 
    Gateway[FTL Gateway<br/>mcp-gateway]
    Auth[FTL Authorizer<br/>mcp-authorizer]
    Tools[Tool Components<br/>WASM]
    
    subgraph Runtime [Spin Runtime - WebAssembly Orchestration]
        subgraph ServiceMesh [*.spin.internal Network<br/>Internal Service Mesh]
            Gateway
            Auth  
            Tools
        end
    end
    
    Client --|HTTP/JSON-RPC| Gateway
    Gateway --|Internal HTTP| Tools
    Auth --|JWT Validation| Gateway
    Client -.->|Bearer Token| Auth
    
    classDef client fill:#e1f5fe
    classDef component fill:#f3e5f5  
    classDef runtime fill:#e8f5e8
    classDef network fill:#fff3e0
    
    class Client client
    class Gateway,Auth,Tools component
    class Runtime runtime
    class ServiceMesh network
```

### Key Architectural Patterns

1. **Configuration Abstraction**: User-friendly `ftl.toml` → Complex `spin.toml` (dynamic transpilation)
2. **Component Isolation**: WebAssembly sandboxing for security and language diversity
3. **Internal Service Mesh**: `*.spin.internal` networking for component communication
4. **Edge-Native Design**: Optimized for global distributed deployment

## Core Components

### 1. MCP Gateway

**Location**: `/components/mcp-gateway/src/gateway.rs`

**Purpose**: Central router providing MCP-compliant access to all tools.

**Key Responsibilities**:
- **Tool Discovery**: `GET http://{component}.spin.internal/` for metadata
- **Request Routing**: `POST http://{component}.spin.internal/{tool}` for execution
- **Schema Validation**: Optional JSON Schema validation via `jsonschema` crate
- **Response Assembly**: MCP-compliant JSON-RPC response formatting
- **Scoped Access**: Path-based component scoping (`/mcp/x/{component}`)

**Architecture Flow**:
```mermaid
flowchart TD
    A[MCP Client Request<br/>JSON-RPC] --> B[Protocol Parser]
    B --> C[Tool Discovery]  
    C --> D[Request Router]
    D --> E[Response Format]
    E --> F[MCP Client Response]
    
    B -.-> B1[Validates JSON-RPC 2.0 format]
    C -.-> C1[Parallel GET requests to components]
    D -.-> D1[POST to specific tool endpoint]
    E -.-> E1[Wrap in MCP content structure]
    
    classDef process fill:#e3f2fd
    classDef detail fill:#f5f5f5
    classDef endpoint fill:#e8f5e8
    
    class B,C,D,E process
    class B1,C1,D1,E1 detail  
    class A,F endpoint
```

### 2. MCP Authorizer

**Location**: `/components/mcp-authorizer/src/lib.rs`

**Purpose**: High-performance JWT authentication gateway securing MCP endpoints.

**Authentication Architecture**:
```mermaid
flowchart TD
    A[Client Request<br/>Bearer Token] --> B[Token Extract]
    B --> C[JWKS Fetch]
    C --> D[JWT Validation]
    D --> E[Context Headers]
    E --> F[Forward to MCP Gateway]
    
    B -.-> B1[Authorization: Bearer {token}]
    C -.-> C1[5-minute cached JWKS endpoint]
    D -.-> D1[Signature, issuer, expiry, scopes]
    E -.-> E1[x-auth-client-id, x-auth-user-id, etc.]
    
    classDef auth fill:#fff3e0
    classDef detail fill:#f5f5f5
    classDef endpoint fill:#e8f5e8
    
    class B,C,D,E auth
    class B1,C1,D1,E1 detail
    class A,F endpoint
```

**Security Features**:
- **JWKS Caching**: 5-minute cache reduces provider API calls
- **OAuth 2.0 Compliance**: RFC8414/RFC9728 endpoint discovery
- **Scope Enforcement**: Per-request scope validation with fallback
- **Tenant Isolation**: Organization-based access control
- **Discovery Endpoints**: OAuth metadata for client registration

**Token Validation Process**:
1. Extract Bearer token from Authorization header
2. Fetch/cache JWKS from configured endpoint
3. Validate JWT signature, issuer, audience, expiration
4. Enforce required scopes (configurable per deployment)
5. Add authentication context headers for downstream components

### 3. Tool Components (WASM)

**Purpose**: Individual business logic implementations running in secure WebAssembly sandboxes.

**Component Structure**:
```
Tool Component (WASM)
├── Business Logic        ← Your tool implementation (any language)
├── MCP Adapter          ← Auto-generated HTTP endpoint handler
├── JSON Schema          ← Auto-generated from type definitions
├── Metadata Endpoint    ← Tool discovery information (/?)
└── Tool Endpoints       ← Actual tool functions (/tool-name)
```

**Multi-Language Support**:
- **Rust**: Native WASM compilation with `wasm32-wasi` target
- **Python**: `componentize-py` generates WASM components from Python
- **Go**: `TinyGo` compiler with WASM Component Model support
- **TypeScript**: `componentize-js` with Jco toolchain

**Runtime Capabilities**:
- **Isolated Execution**: Separate WASM linear memory per component
- **Outbound Network Access**: Whitelist-controlled HTTP requests
- **Internal Communication**: Access to other components via `*.spin.internal`

### 4. Spin Framework Integration

**Purpose**: WebAssembly runtime orchestration and internal networking.

**Runtime Architecture**:
```mermaid
graph TB
    External[External Web<br/>Public endpoint access]
    
    subgraph SpinRuntime [Spin Runtime - Wasmtime-based]
        subgraph Components [WASM Components]
            Gateway[MCP Gateway<br/>public]
            Auth[Authorizer<br/>public] 
            Tool1[Tool 1<br/>private]
            Tool2[Tool N<br/>private]
        end
        
        subgraph ServiceMesh [*.spin.internal Networking<br/>In-Process Service Mesh]
            Gateway
            Auth
            Tool1
            Tool2
        end
        
        subgraph HttpTriggers [HTTP Trigger Routing]
            Gateway
            Auth
        end
    end
    
    External --> HttpTriggers
    Gateway -.->|Internal HTTP| Tool1
    Gateway -.->|Internal HTTP| Tool2
    Auth -.->|Validation| Gateway
    
    classDef external fill:#e3f2fd
    classDef public fill:#fff3e0
    classDef private fill:#f3e5f5
    classDef infrastructure fill:#e8f5e8
    
    class External external
    class Gateway,Auth public
    class Tool1,Tool2 private
    class SpinRuntime,ServiceMesh,HttpTriggers infrastructure
```

**Key Features**:
- **Internal Service Mesh**: `http://*.spin.internal` domains for component communication
- **HTTP Triggers**: Public endpoint routing based on component configuration

## Configuration System

FTL's configuration system uses a sophisticated **dynamic transpilation** approach where simple user configurations expand into complex runtime configurations.

### Configuration Layers

```mermaid
flowchart TD
    A[ftl.toml<br/>Simple, Human-Friendly<br/>User Layer] 
    B[Dynamic Transpilation<br/>Process]
    C[spin.toml<br/>Complex, Generated<br/>Runtime Layer]
    D[Spin Framework<br/>Processing]
    E[WASM Components + HTTP Routing<br/>Execution Layer]
    
    A -->|User Configuration| B
    B -->|Generated Configuration| C
    C -->|Runtime Processing| D
    D -->|Deployed Components| E
    
    A -.-> A1[Project root<br/>Version controlled<br/>User edited]
    C -.-> C1[Temporary directory<br/>Auto-generated<br/>Never edited]
    E -.-> E1[Running application<br/>Live components<br/>HTTP endpoints]
    
    classDef userLayer fill:#e1f5fe
    classDef process fill:#fff3e0
    classDef runtimeLayer fill:#f3e5f5
    classDef execution fill:#e8f5e8
    classDef detail fill:#f5f5f5
    
    class A userLayer
    class B,D process
    class C runtimeLayer
    class E execution
    class A1,C1,E1 detail
```

### FTL Configuration (`ftl.toml`)

**Location**: Project root
**Nature**: Persistent, version-controlled, user-edited

```toml
[project]
name = "my-project"
access_control = "private"  # Triggers auth system

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = true

[oauth]  # Custom OAuth provider
issuer = "https://auth.example.com"
audience = "api-identifier"

[variables]
custom_api_key = { default = "demo-key" }

[tools.text-analyzer]
path = "components/text-analyzer"
allowed_outbound_hosts = ["https://api.openai.com"]
```

### Spin Configuration (`spin.toml`)

**Location**: Temporary directory (`/tmp/ftl-XXXXXX/`)
**Nature**: Generated, ephemeral, never edited by users

**Generation Process** (`/crates/commands/src/config/transpiler.rs`):

1. **Parse** `ftl.toml` configuration
2. **Generate System Variables**: 15+ auth-related variables from `access_control = "private"`
3. **Component Resolution**: Add gateway/authorizer components from registry
4. **HTTP Trigger Configuration**: Dynamic routing based on authentication mode
5. **Variable Injection**: Template variables into component configurations
6. **Path Resolution**: Handle local vs registry component references

**Example Variable Expansion**:
```toml
# Input: Single line in ftl.toml
access_control = "private"

# Generated: 15+ variables in spin.toml
[variables]
auth_enabled = { default = "true" }
mcp_jwt_issuer = { default = "https://divine-lion-50-staging.authkit.app" }
mcp_jwt_audience = { default = "https://mcp-server" }
mcp_jwt_jwks_uri = { default = "https://divine-lion-50-staging.authkit.app/.well-known/jwks.json" }
mcp_authorization_header_name = { default = "Authorization" }
mcp_gateway_url = { default = "http://ftl-mcp-gateway.spin.internal" }
# ... 10+ more variables
```

### Configuration Transpilation Patterns

**Authentication Mode Detection**:
```rust
// From transpiler.rs
match ftl_config.project.access_control.as_deref() {
    Some("private") => {
        // Generate 15+ auth variables
        // Add authorizer component
        // Configure authenticated HTTP triggers
    }
    _ => {
        // Public mode - direct gateway access
        // Simpler HTTP trigger configuration
    }
}
```

**Component Resolution**:
```rust
// Local component
[tools.my-tool]
path = "components/my-tool"

// Becomes:
[[component]]
id = "my-tool"
source = { url = "file:components/my-tool/target/wasm32-wasi/release/my-tool.wasm" }

// Registry component  
[tools.json-formatter]
# Omitted path = registry component

// Becomes:
[[component]]
id = "json-formatter"
source = { url = "ghcr.io/fastertools/ftl-tool-json-formatter:latest" }
```

## SDK Architecture

FTL provides language-specific SDKs that abstract away the complexity of MCP protocol implementation and WebAssembly compilation.

### Design Philosophy

**Thin SDKs**: Each SDK provides minimal abstraction over the MCP protocol, focusing on:
- Type definitions for MCP protocol structures
- Convenient builders for tool responses
- Language-specific ergonomics (decorators, macros, etc.)
- Schema generation from native type systems

**No HTTP Logic**: SDKs contain no HTTP server code - that's handled by generated WASM adapters.

### Available SDKs

For detailed usage examples, API documentation, and getting started guides, see each SDK's individual README file.

- [Rust SDK](../../sdk/rust/README.md)
- [Python SDK](../../sdk/python/README.md)
- [Go SDK](../../sdk/go/README.md)
- [Typescript SDK](../../sdk/typescript/README.md)


## Request Flow Deep Dive

Let's trace a complete request from an MCP client through the entire FTL stack:

### 1. Client Request

**MCP Protocol**: JSON-RPC 2.0 over HTTP/WebSocket

```json
POST /mcp
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGci...

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "text-analyzer__analyze-sentiment",
    "arguments": {
      "content": "Hello, world! This is a great day.",
      "options": { "include_confidence": true }
    }
  }
}
```

### 2. Authentication (Private Projects)

**Authorizer Processing**:

```
Bearer Token Extraction
    ↓
JWKS Endpoint Fetch (cached 5 min)
    ↓ 
JWT Validation:
- Signature verification (RS256/ES256)
- Issuer validation (iss claim)
- Audience validation (aud claim)
- Expiration check (exp claim)
- Required scope check (scope claim)
    ↓
Add Context Headers:
- x-auth-client-id: app-client-123
- x-auth-user-id: user-456
- x-auth-issuer: https://auth.example.com
- x-auth-scopes: mcp:read mcp:write
- x-auth-org-id: org-789 (if tenant isolation enabled)
    ↓
Forward to Gateway: POST http://ftl-mcp-gateway.spin.internal/mcp
```

### 3. Gateway Processing  

**Tool Discovery & Routing**:

```
JSON-RPC Validation
    ↓
Parse Tool Name: "text-analyzer__analyze-sentiment"
- Component: "text-analyzer"  
- Tool: "analyze-sentiment"
    ↓
Component Discovery (parallel):
GET http://text-analyzer.spin.internal/ 
← Returns: {
  "tools": [
    {
      "name": "analyze-sentiment",
      "description": "Analyze text sentiment",
      "parameters": { /* JSON Schema */ }
    }
  ]
}
    ↓
Argument Validation (optional):
- Validate against JSON schema
- Type coercion where possible
    ↓
Tool Invocation Routing
```

### 4. Component Execution

**WASM Component Call**:

```
POST http://text-analyzer.spin.internal/analyze-sentiment
Content-Type: application/json
x-auth-client-id: app-client-123
x-auth-user-id: user-456
x-auth-scopes: mcp:read mcp:write

{
  "content": "Hello, world! This is a great day.",
  "options": { "include_confidence": true }
}
    ↓
WASM Component (text-analyzer):
1. HTTP handler receives request
2. Deserialize parameters
3. Execute business logic:
   - Sentiment analysis algorithm
   - Confidence calculation
4. Serialize response
    ↓
Component Response:
{
  "sentiment": "positive",
  "confidence": 0.92,
  "keywords": ["great", "day"]
}
```

### 5. Response Assembly

**Gateway Response Formatting**:

```json
{
  "jsonrpc": "2.0", 
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"sentiment\": \"positive\", \"confidence\": 0.92, \"keywords\": [\"great\", \"day\"]}"
      }
    ]
  }
}
```
