# FTL Architecture

## Philosophy: Polyglot by Design

FTL embodies a **"best tool for the job"** philosophy. We don't force a single language or framework - instead, we carefully select technologies based on their strengths:

- **Go for CLI**: Excellent ergonomics for command-line tools, great cross-platform support, single binary distribution
- **Rust for WebAssembly Components**: Superior WASM compilation, memory safety, performance
- **Multiple SDKs**: Meet developers in their preferred language (Python, TypeScript, Go, Rust)

This polyglot approach is enabled by WebAssembly and the Component Model, which provide language-agnostic interfaces and secure sandboxing.

## Project Structure

```
ftl/
├── cmd/ftl/                 # Go CLI entry point
│   └── main.go             # Main application
│
├── internal/               # Private Go packages (CLI implementation)
│   ├── api/               # FTL Engine API client
│   ├── auth/              # Authentication and credential management
│   ├── cli/               # Command implementations
│   ├── console/           # Web console handlers and logic
│   ├── handlers/          # HTTP request handlers
│   ├── mcp/               # MCP server implementation
│   ├── polling/           # Real-time polling and state management
│   ├── scaffold/          # Project scaffolding and templates
│   ├── state/             # Application state management
│   └── synthesis/         # Configuration synthesis (YAML/JSON/CUE → Spin)
│
├── components/             # WebAssembly MCP components (Rust)
│   ├── mcp-authorizer/    # Authorization component for MCP servers
│   └── mcp-gateway/       # Gateway component for MCP servers
│
├── mcp-server/             # Standalone MCP server for ftl CLI integration
│   ├── main.go            # MCP server entry point
│   └── tools/             # MCP tool implementations
│
├── sdk/                    # Multi-language SDKs for building MCP tools
│   ├── rust/              # Rust SDK
│   ├── rust-macros/       # Rust procedural macros
│   ├── python/            # Python SDK
│   ├── typescript/        # TypeScript SDK
│   └── go/                # Go SDK
│
├── web-templates/          # Templ web UI templates for console dashboard
├── static/                 # Static web assets (logos, CSS, JS)
├── e2e-tests/              # End-to-end browser tests (Playwright)
│   └── e2e/
│       ├── specs/         # Test specifications
│       ├── pages/         # Page object models
│       └── utils/         # Test utilities
│
├── examples/               # Example applications and use cases
├── docs/                   # Documentation
│
├── package.json            # Node.js dependencies for browser testing
├── playwright.config.js    # Playwright test configuration
├── go.mod                  # Go module definition (single module)
├── Cargo.toml             # Rust workspace configuration
└── Makefile               # Build orchestration (polyglot entry point)
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
- Web-based development console with real-time project management
- MCP server integration for AI agent tooling

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

### 4. Development Console (Web UI)

The FTL CLI includes a web-based development console for project management:

**Technology Stack:**
- **Templ**: Type-safe Go HTML templating
- **HTMX**: Dynamic UI interactions without JavaScript frameworks
- **Go HTTP Server**: Integrated web server with real-time polling

**Features:**
- Multi-project management with switching
- Real-time command execution and output streaming
- Live log monitoring with auto-refresh
- Project status tracking and build management
- Integration with MCP server for AI agent tooling

**Benefits:**
- **Zero Dependencies**: No Node.js build pipeline required
- **Type Safety**: Templ provides compile-time HTML validation
- **Performance**: Server-side rendering with minimal client-side JavaScript
- **Integration**: Direct access to FTL CLI functionality

### 5. MCP Server Integration

Standalone MCP (Model Context Protocol) server for AI agent integration:

**Purpose:**
- Exposes FTL CLI functionality as MCP tools
- Enables AI agents to manage FTL projects
- Provides structured interface for automation

**Key Tools:**
- Project lifecycle management (build, up, logs)
- Status monitoring and health checks
- Command execution with structured responses

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

### Why Templ + HTMX for the Console?

1. **Type Safety**: Templ provides compile-time HTML validation within Go
2. **Zero Build Pipeline**: No Node.js, webpack, or complex frontend tooling
3. **Performance**: Server-side rendering with minimal client-side JavaScript
4. **Simplicity**: HTMX enables dynamic UIs with HTML attributes
5. **Integration**: Direct access to Go CLI functionality without APIs

### Why Separate Directory Structure?

1. **Clarity**: `/e2e-tests/` clearly indicates browser tests, not unit tests
2. **Purpose**: `/web-templates/` distinguishes UI templates from scaffolding templates
3. **Organization**: `/mcp-server/` separates standalone MCP server from CLI
4. **Maintainability**: Clear separation of concerns reduces cognitive load

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

### For MCP Tool Development:
1. **Choose Language**: Select SDK based on use case
2. **Write Tool**: Implement MCP tool using SDK
3. **Build**: Compile to WebAssembly component
4. **Compose**: Combine multiple tools into single MCP server
5. **Deploy**: Run locally or deploy to edge network

### For FTL CLI Development:
1. **Setup**: Run `make setup-all` for complete environment setup
2. **Development**: Use `make dev` for quick build and test cycle
3. **Console Testing**: Use `ftl dev console` for web UI development
4. **Integration Testing**: Run `make test-browser` for end-to-end tests
5. **Full Testing**: Run `make test` for comprehensive test suite

### Testing Strategy:
- **Unit Tests**: Go and Rust tests for individual components
- **Integration Tests**: MCP server functionality testing
- **E2E Tests**: Playwright tests for web console functionality
- **Component Tests**: WebAssembly component testing with Spin

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
4. **Console Features**: Add new web UI capabilities
5. **MCP Tools**: Extend MCP server with new tool integrations
6. **Synthesis Formats**: Support additional configuration formats
7. **Testing**: Add new test scenarios for browser or integration testing

### Key Files for Contributors:
- `Makefile`: Build orchestration and task automation
- `internal/cli/`: CLI command implementations
- `web-templates/`: Web console UI templates
- `mcp-server/tools/`: MCP tool implementations
- `e2e-tests/`: Browser testing specifications
- `components/`: WebAssembly component development

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development setup and guidelines.