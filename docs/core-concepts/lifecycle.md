# Project Lifecycle

Understanding what happens behind FTL's commands helps you debug issues, optimize your workflow, and extend FTL's capabilities. This guide explains the complete project lifecycle from initialization to deployment.

## Overview

FTL's project lifecycle follows a clear progression:

```
ftl init → ftl add → ftl build → ftl up → ftl deploy
   │         │         │          │         │
   ▼         ▼         ▼          ▼         ▼
Project   Tools    WASM       Local    Production
Setup     Added   Built      Server   Deployment
```

## `ftl init`: Project Initialization

### What It Does
Creates a new FTL project with the foundational configuration and directory structure.

### Behind the Scenes

1. **Directory Creation**:
   ```bash
   mkdir my-project
   cd my-project
   ```

2. **Configuration Generation**:
   Creates `ftl.toml` with project metadata:
   ```toml
   [project]
   name = "my-project"
   access_control = "public"  # or "private" with --private flag
   
   [mcp]
   gateway = "ghcr.io/fastertools/mcp-gateway:latest"
   authorizer = "ghcr.io/fastertools/mcp-authorizer:latest"
   validate_arguments = true
   ```

3. **Template Processing**:
   - Downloads the `ftl-mcp-server` Spin template if not cached
   - Processes template variables like `{{project-name}}`
   - Creates initial directory structure and README

4. **Spin Configuration**:
   Generates initial `spin.toml` with base configuration:
   ```toml
   spin_manifest_version = "2"
   name = "my-project"
   
   [variables]
   tool_components = { default = "" }  # Empty until tools added
   
   [[component]]
   id = "mcp-gateway"
   source = { url = "ghcr.io/fastertools/mcp-gateway:latest" }
   ```

### File System Result
```
my-project/
├── ftl.toml          # FTL project configuration
├── spin.toml         # Spin application configuration
├── components/       # Directory for tool components (empty)
└── README.md         # Getting started guide
```

### Command Variants
- `ftl init my-project`: Public access (no authentication)
- `ftl init my-project --private`: Private access (JWT authentication enabled)
- `ftl init my-project --template custom`: Use custom project template

## `ftl add`: Tool Addition

### What It Does
Adds a new tool component to your project in a specific programming language.

### Behind the Scenes

1. **Template Resolution**:
   ```bash
   # Resolves language to specific template
   --language rust    → ftl-mcp-rust
   --language python  → ftl-mcp-python
   --language go      → ftl-mcp-go
   --language typescript → ftl-mcp-ts
   ```

2. **Template Download**:
   - Checks local cache: `~/.cache/spin/templates/`
   - Downloads from registry if not cached or outdated
   - Verifies template integrity

3. **Component Generation**:
   ```bash
   # Creates component directory
   mkdir components/my-tool
   cd components/my-tool
   
   # Processes template with variables
   {{tool-name}} → my-tool
   {{component-name}} → my_tool (snake_case)
   ```

4. **Language-Specific Setup**:

   **Rust**:
   ```toml
   # Cargo.toml
   [package]
   name = "my-tool"
   
   [dependencies]
   ftl-sdk = "0.1.0"
   serde = { version = "1.0", features = ["derive"] }
   
   [lib]
   crate-type = ["cdylib"]
   
   [[bin]]
   name = "my-tool"
   path = "src/main.rs"
   ```

   **Python**:
   ```toml
   # pyproject.toml
   [project]
   name = "my-tool"
   dependencies = ["ftl-sdk"]
   
   [build-system]
   requires = ["componentize-py"]
   ```

5. **Configuration Updates**:
   Updates `ftl.toml`:
   ```toml
   [tools.my-tool]
   path = "components/my-tool"
   allowed_outbound_hosts = []
   ```

   Updates `spin.toml` via transpilation:
   ```toml
   [variables]
   tool_components = { default = "my-tool" }  # Updated list
   
   [[component]]
   id = "my-tool"
   source = "components/my-tool/target/wasm32-wasip1/release/my-tool.wasm"
   ```

### Generated Component Structure

**Rust**:
```
components/my-tool/
├── Cargo.toml        # Rust package configuration
├── src/
│   ├── lib.rs        # Tool implementation with ftl-sdk
│   └── main.rs       # WASM binary entry point
├── README.md         # Tool documentation
└── Makefile          # Build automation
```

**Python**:
```
components/my-tool/
├── pyproject.toml    # Python package configuration
├── src/
│   └── __init__.py   # Tool implementation
├── README.md         # Tool documentation
└── Makefile          # Build automation with componentize-py
```

## `ftl build`: Compilation to WebAssembly

### What It Does
Compiles all tool components to WebAssembly modules and prepares the complete application.

### Behind the Scenes

1. **Configuration Transpilation**:
   - Reads `ftl.toml` configuration
   - Generates up-to-date `spin.toml` with current tools
   - Resolves component dependencies and versions

2. **Parallel Component Building**:
   ```bash
   # For each tool component in parallel
   for component in tools:
       execute_build_command(component)
   ```

3. **Language-Specific Build Process**:

   **Rust**:
   ```bash
   cd components/my-rust-tool
   cargo build --target wasm32-wasip1 --release
   # Produces: target/wasm32-wasip1/release/my-rust-tool.wasm
   ```

   **Python**:
   ```bash
   cd components/my-python-tool
   componentize-py componentize guest -o my-python-tool.wasm
   # Produces: my-python-tool.wasm
   ```

   **Go**:
   ```bash
   cd components/my-go-tool
   tinygo build -o my-go-tool.wasm -target wasip1 .
   # Produces: my-go-tool.wasm
   ```

4. **Schema Generation**:
   - Rust: Automatic via `schemars` derive macros
   - Python: Via type hints and `pydantic`
   - Go: Via struct tags and reflection
   - Generates JSON Schema files for MCP tool descriptions

5. **Component Validation**:
   - Verifies WASM modules are valid
   - Checks Component Model compatibility
   - Validates generated schemas

6. **External Component Resolution**:
   Downloads external components if needed:
   ```bash
   # Downloads mcp-gateway and mcp-authorizer if not cached
   crane pull ghcr.io/fastertools/mcp-gateway:latest
   crane pull ghcr.io/fastertools/mcp-authorizer:latest
   ```

### Build Output
```
my-project/
├── components/
│   └── my-tool/
│       ├── target/wasm32-wasip1/release/my-tool.wasm  # WASM binary
│       └── my-tool.schema.json                       # Generated schema
├── spin.toml         # Updated with component paths
└── .spin/            # Spin cache directory
    └── components/   # Cached external components
```

### Build Performance Optimizations
- **Parallel Compilation**: All components build simultaneously
- **Incremental Builds**: Only recompiles changed components
- **Component Caching**: External components cached locally
- **Schema Caching**: Regenerated only when tool interfaces change

## `ftl up`: Local Development Server

### What It Does
Starts a local Spin server running your complete MCP application.

### Behind the Scenes

1. **Pre-flight Checks**:
   - Verifies `spin.toml` exists and is valid
   - Checks all WASM components are built and accessible
   - Validates component compatibility

2. **Spin Server Initialization**:
   ```bash
   # Internal command executed
   spin up --listen 127.0.0.1:3000
   ```

3. **Component Loading**:
   ```
   Loading Components:
   ├── mcp-gateway (public endpoint)
   ├── mcp-authorizer (if auth enabled)
   └── Tool Components:
       ├── my-rust-tool.wasm
       ├── my-python-tool.wasm
       └── my-go-tool.wasm
   ```

4. **Runtime Architecture Setup**:
   ```
   ┌─────────────────────────────────────┐
   │         Spin Runtime                │
   ├─────────────────────────────────────┤
   │ HTTP Server (0.0.0.0:3000)         │
   │  ├─ /tools/list                     │
   │  ├─ /tools/call                     │
   │  └─ /ping                           │
   ├─────────────────────────────────────┤
   │ Internal Network                    │
   │  ├─ mcp-gateway.spin.internal       │
   │  ├─ mcp-authorizer.spin.internal    │
   │  └─ my-tool.spin.internal           │
   └─────────────────────────────────────┘
   ```

5. **Service Registration**:
   - Each component registers its internal endpoints
   - Gateway component discovers available tools
   - Health checks establish component readiness

6. **Development Features**:
   - **Auto-reload**: File changes trigger automatic rebuilds
   - **Live Logging**: Component stdout/stderr in real-time
   - **Error Reporting**: Detailed error messages with stack traces

### Server Output
```bash
✅ FTL server started successfully
🌐 MCP server available at: http://localhost:3000
📋 Available tools:
   - my-rust-tool/analyze_text
   - my-python-tool/process_data
   - my-go-tool/calculate_metrics
🔄 Watching for changes...
```

### Runtime Behavior
- **Hot Reload**: Changes to tool code trigger automatic rebuilds
- **Concurrent Requests**: Multiple tool calls handled simultaneously  
- **Error Isolation**: Tool failures don't crash the server
- **Resource Management**: Memory and CPU usage monitoring

## `ftl deploy`: Production Deployment

### What It Does
Deploys your FTL application to production infrastructure.

### Behind the Scenes

1. **Build Verification**:
   ```bash
   # Ensures latest build
   ftl build --release
   
   # Validates all components
   validate_deployment_readiness()
   ```

2. **Bundle Preparation**:
   - Creates deployment artifact with all components
   - Includes configuration and dependencies
   - Optimizes WASM modules for production

3. **Infrastructure Provisioning** (FTL Engine):
   - Creates managed infrastructure resources
   - Sets up load balancing and auto-scaling
   - Configures monitoring and logging

4. **Component Deployment**:
   ```
   Upload Pipeline:
   ├── Application Bundle (spin.toml + components)
   ├── Environment Configuration
   ├── SSL/TLS Certificates
   └── Health Check Endpoints
   ```

5. **Service Activation**:
   - Deploys to staging environment first
   - Runs health checks and integration tests
   - Promotes to production with zero-downtime cutover

6. **DNS Configuration**:
   ```
   Production Endpoints:
   ├── https://my-project.ftlengine.dev  # Auto-generated
   ├── https://custom-domain.com        # Custom domain (optional)
   └── Health: https://my-project.ftlengine.dev/_health
   ```

## Configuration Evolution

Throughout the lifecycle, configurations evolve:

### Initial State (after `ftl init`)
```toml
[project]
name = "my-project"

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:latest"

# No tools yet
```

### After Adding Tools
```toml
[project]
name = "my-project"

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:latest"

[tools.text-analyzer]
path = "components/text-analyzer"

[tools.data-processor]  
path = "components/data-processor"
allowed_outbound_hosts = ["https://api.openai.com"]
```

### Production Configuration
```toml
[project]
name = "my-project"
access_control = "private"

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.1.0"  # Pinned version
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.1.0"
validate_arguments = true

[oidc]
issuer = "https://auth.company.com"
audience = "my-project-api"

[tools.text-analyzer]
path = "components/text-analyzer"
environment_variables = { "MODEL_VERSION" = "v2" }

[tools.data-processor]
path = "components/data-processor"
allowed_outbound_hosts = ["https://api.openai.com"]
```

## Debugging the Lifecycle

### Common Issues and Solutions

**Build Failures**:
```bash
# Check component-specific build logs
ftl build --verbose

# Test individual component builds
cd components/my-tool && make build
```

**Runtime Errors**:
```bash  
# Detailed logging during development
ftl up --log-level debug

# Component-specific logs
spin logs --component my-tool
```

**Deployment Issues**:
```bash
# Validate deployment readiness
ftl build --validate

# Check deployment status
ftl eng status my-project
```

## Performance Considerations

### Build Optimization
- **Dependency Caching**: Reuse compiled dependencies
- **Incremental Compilation**: Only rebuild changed components
- **Parallel Processing**: Utilize multiple CPU cores
- **Size Optimization**: Strip debug symbols in release builds

### Runtime Optimization
- **Component Startup**: WASM modules start in ~1-5ms
- **Memory Usage**: Linear memory model with precise control
- **Network Efficiency**: Internal component communication via shared memory
- **Auto-scaling**: Production deployments scale based on load

## Next Steps

Now that you understand the complete lifecycle:
- **Apply Your Knowledge**: Try the [Getting Started Tutorials](../getting-started/)
- **Solve Specific Problems**: Check [How-to Guides](../guides/)
- **Reference APIs**: Browse [SDK Reference](../sdk-reference/)
- **Advanced Patterns**: Explore [Examples](../../examples/)

Understanding the lifecycle helps you work more effectively with FTL, debug issues faster, and optimize your development workflow.