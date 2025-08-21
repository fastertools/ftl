# FTL Architecture

## Philosophy: Polyglot by Design

FTL embodies a **"best tool for the job"** philosophy. We don't force a single language or framework - instead, we carefully select technologies based on their strengths:

- **Go for CLI**: Excellent ergonomics for command-line tools, great cross-platform support, single binary distribution
- **Rust for WebAssembly Components**: Superior WASM compilation, memory safety, performance
- **Multiple SDKs**: Meet developers in their preferred language (Python, TypeScript, Go, Rust)

This polyglot approach is enabled by WebAssembly and the Component Model, which provide language-agnostic interfaces and secure sandboxing.

## Project Structure

```
ftl-cli/
├── cmd/ftl/                 # Go CLI entry point
│   └── main.go             # Main application
│
├── internal/               # Private Go packages (CLI implementation)
│   ├── api/               # FTL Engine API client
│   ├── auth/              # Authentication and credential management
│   ├── cli/               # Command implementations
│   ├── ftl/               # Core FTL business logic
│   ├── scaffold/          # Project scaffolding and templates
│   └── synthesis/         # Configuration synthesis (YAML/JSON/CUE → Spin)
│
├── pkg/                    # Public Go packages (can be imported by other projects)
│   ├── types/             # Shared data structures and manifest types
│   └── spin/              # Spin framework executor and utilities
│
├── components/             # WebAssembly MCP components (Rust)
│   ├── mcp-authorizer/    # Authorization component for MCP servers
│   └── mcp-gateway/       # Gateway component for MCP servers
│
├── sdk/                    # Multi-language SDKs for building MCP tools
│   ├── rust/              # Rust SDK
│   ├── rust-macros/       # Rust procedural macros
│   ├── python/            # Python SDK
│   ├── typescript/        # TypeScript SDK
│   └── go/                # Go SDK
│
├── templates/              # Quick-start templates for new projects
├── examples/               # Example applications and use cases
├── docs/                   # Documentation
│
├── go.mod                  # Go module definition (single module)
├── Cargo.toml             # Rust workspace configuration
└── Makefile               # Build orchestration
```

## Component Architecture

### 1. FTL CLI (Go)

The CLI is the primary user interface for FTL. Written in Go for:
- Excellent CLI libraries (Cobra, Viper)
- Single binary distribution
- Great cross-platform support
- Fast compilation
- Rich ecosystem for cloud/DevOps tools

**Key Responsibilities:**
- Project initialization and scaffolding
- Building and testing MCP tools
- Deploying to FTL Engine or other platforms
- Managing authentication and credentials
- Synthesizing configurations (YAML/JSON/CUE → Spin manifests)

### 2. WebAssembly Components (Rust)

MCP server components are written in Rust and compiled to WebAssembly for:
- Memory safety and performance
- Excellent WebAssembly support
- Small binary sizes
- Predictable resource usage

**Components:**
- **mcp-authorizer**: Handles MCP authorization flows
- **mcp-gateway**: Routes requests to appropriate MCP tools

### 3. SDKs (Polyglot)

SDKs enable developers to build MCP tools in their preferred language:

- **Rust SDK**: For performance-critical tools
- **Python SDK**: For AI/ML integrations and data processing
- **TypeScript SDK**: For web developers and Node.js ecosystems
- **Go SDK**: For cloud-native and DevOps tools

Each SDK compiles to WebAssembly using language-specific toolchains.

## Key Design Decisions

### Why Go for the CLI?

1. **Developer Experience**: Go's simplicity makes the CLI codebase approachable
2. **Distribution**: Single binary with no runtime dependencies
3. **Ecosystem**: Rich libraries for HTTP, cloud APIs, and DevOps tools
4. **Performance**: Fast startup times crucial for CLI tools
5. **Cross-platform**: Excellent support for Windows, macOS, and Linux

### Why Keep Rust for Components?

1. **WebAssembly Excellence**: Rust has the most mature WASM toolchain
2. **Safety**: Memory safety without garbage collection
3. **Performance**: Near-native speed in WebAssembly
4. **Component Model**: First-class support for WASM Component Model
5. **Size**: Produces small, efficient WASM modules

### Why Multiple SDKs?

1. **Developer Choice**: Meet developers where they are
2. **Ecosystem Integration**: Each language brings its own libraries
3. **Use Case Optimization**: Different languages excel at different tasks
4. **WebAssembly Compatibility**: All compile to WASM via Component Model

## Data Flow

```
User → FTL CLI (Go) → Configuration Files (YAML/JSON/CUE)
                    ↓
              Synthesis Engine
                    ↓
              Spin Manifest
                    ↓
        WebAssembly Components (Built)
                    ↓
         Deployment (Local/Cloud/Edge)
```

## Security Model

1. **WebAssembly Sandboxing**: Each component runs in isolation
2. **Capability-Based Security**: Components only access granted resources
3. **Secure Credential Storage**: OS keyring integration for sensitive data
4. **MCP Authorization**: Built-in auth components for secure access

## Development Workflow

1. **Choose Language**: Select SDK based on use case
2. **Write Tool**: Implement MCP tool using SDK
3. **Build**: Compile to WebAssembly component
4. **Compose**: Combine multiple tools into single MCP server
5. **Deploy**: Run locally or deploy to edge network

## Future Directions

- **More SDKs**: Java, C#, Swift support planned
- **Enhanced Composition**: Visual tool composition interface
- **Registry Integration**: Public registry for sharing MCP tools
- **Performance Optimizations**: Further cold start improvements
- **Extended Platform Support**: More deployment targets

## Contributing

This architecture is designed to be extensible. Key extension points:

1. **New SDKs**: Add support for additional languages
2. **New Components**: Build reusable MCP components
3. **CLI Commands**: Extend CLI with new functionality
4. **Synthesis Formats**: Support additional configuration formats

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.