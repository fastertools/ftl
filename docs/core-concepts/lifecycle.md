# Project Lifecycle

Understanding FTL's dynamic configuration system is essential for debugging, optimizing workflows, and extending capabilities. This guide explains the complete project lifecycle with a focus on how FTL's transpilation engine converts your simple `ftl.toml` configuration into complete Spin applications.

## Overview

One of fastertools's key features is **dynamic configuration transpilation**. While you work with a simple `ftl.toml` file, fastertools automatically generates complex `spin.toml` configurations on-demand for each operation:

```
User writes ftl.toml → FTL transpiles → Temporary spin.toml → Spin executes
       ↓                    ↓                  ↓              ↓
   Simple config      Dynamic generation   Runtime config   WASM execution
```

**Key Concepts:**
- **ftl.toml** - Your project configuration:  persistent, version-controlled(or at least it SHOULD be!)
- **spin.toml** - Runtime configuration (temporary, auto-generated) 
- **Transpilation** - Dynamic conversion that happens during every command

## Core Architecture: Configuration Transpilation

### The fastertools Configuration Model

fastertools uses a **two-layer configuration architecture**:

1. **User Layer (ftl.toml)** - Simple, declarative configuration
2. **Runtime Layer (spin.toml)** - Complex, executable configuration

```toml
# ftl.toml - What you write
[project]
name = "my-app"
access_control = "private"

[tools.calculator]
path = "components/calculator"
```

Gets transpiled into:

```toml
# Generated spin.toml - What Spin executes
spin_manifest_version = "2"
name = "my-app"

[variables]
tool_components = { default = "calculator" }
auth_enabled = { default = "true" }
mcp_gateway_url = { default = "http://ftl-mcp-gateway.spin.internal/mcp-internal" }
mcp_tenant_id = { required = true }
# ... 15+ more auth variables

[[component]]
id = "mcp"
source = { url = "ghcr.io/fastertools/mcp-authorizer:0.0.13" }

[[component]] 
id = "ftl-mcp-gateway"
source = { url = "ghcr.io/fastertools/mcp-gateway:0.0.15" }

[[component]]
id = "calculator"
source = "components/calculator/target/wasm32-wasip1/release/calculator.wasm"

[[trigger.http]]
route = "/..."
component = "mcp"
```

### Transpilation Process

Every FTL command performs these steps:

1. **Read ftl.toml** - Parse user configuration
2. **Generate Variables** - Inject system variables based on configuration
3. **Resolve Components** - Add gateway, authorizer, and tool components  
4. **Create Triggers** - Set up HTTP routing based on authentication
5. **Write Temporary spin.toml** - Save to temp directory
6. **Execute Spin** - Run command with generated configuration
7. **Cleanup** - Remove temporary files

### Variable Injection System

FTL automatically injects system variables based on your configuration:

**From Simple Access Control:**
```toml
# ftl.toml
access_control = "private"
```

**To Complex Auth Variables:**
```toml
# Generated variables
auth_enabled = { default = "true" }
mcp_gateway_url = { default = "http://ftl-mcp-gateway.spin.internal/mcp-internal" }
mcp_tenant_id = { required = true }
mcp_trace_header = { default = "x-trace-id" }
mcp_provider_type = { default = "ftl_authkit" }
mcp_jwt_issuer = { default = "https://divine-lion-50-staging.authkit.app" }
mcp_jwt_audience = { default = "ftl-mcp-server" }
mcp_jwt_required_scopes = { default = "" }
# ... plus OAuth endpoints, JWKS URIs, etc.
```

## Command Lifecycle

### `ftl init`: Project Foundation

**What It Does:**
Creates a new FTL project with foundational configuration.

**Transpilation Role:**
None - Only creates the initial `ftl.toml` file.

**Behind the Scenes:**

1. **Directory Creation:**
   ```bash
   mkdir my-project && cd my-project
   ```

2. **Template Download:**
   - Downloads `ftl-mcp-server` Spin template if not cached
   - Processes template variables like `{{project-name}}`

3. **ftl.toml Generation:**
   ```toml
   [project]
   name = "my-project"
   access_control = "public"  # or "private" with --private
   
   [mcp]
   gateway = "ghcr.io/fastertools/mcp-gateway:latest"
   authorizer = "ghcr.io/fastertools/mcp-authorizer:latest"
   ```

**File System Result:**
```
my-project/
├── ftl.toml          # FTL project configuration (persistent)
├── components/       # Directory for tool components (empty initially)
└── README.md         # Getting started guide
```

**Note:** No `spin.toml` is created - this is generated dynamically later.

### `ftl add`: Tool Integration

**What It Does:**
Adds a new tool component in a specific programming language.

**Transpilation Role:**
Generates temporary `spin.toml` for Spin template operations.

**Behind the Scenes:**

1. **Transpilation for Template Operations:**
   ```rust
   let temp_spin_toml = generate_temp_spin_toml(&GenerateSpinConfig {
       file_system: &deps.file_system,
       project_path: &working_path,
       download_components: false,  // Not needed for templates
       validate_local_auth: false,  // Not needed for templates
   })?;
   ```

2. **Template Resolution:**
   ```bash
   --language rust → ftl-mcp-rust template
   --language python → ftl-mcp-python template  
   --language go → ftl-mcp-go template
   ```

3. **Component Generation:**
   Uses Spin templates with temporary spin.toml to generate tool structure

4. **ftl.toml Update:**
   ```toml
   [tools.my-tool]
   path = "components/my-tool"
   allowed_outbound_hosts = []
   ```

5. **Cleanup:**
   Temporary `spin.toml` is automatically removed

**File System Changes:**
```
my-project/
├── ftl.toml          # Updated with new tool config
├── components/
│   └── my-tool/      # New tool component directory
│       ├── Cargo.toml (or pyproject.toml, go.mod)
│       ├── src/
│       └── README.md
└── README.md
```

### `ftl build`: Component Compilation

**What It Does:**
Compiles all tool components to WebAssembly and prepares the complete application.

**Transpilation Role:**  
Generates temporary `spin.toml` to discover components and their build configurations.

**Behind the Scenes:**

1. **Dynamic Transpilation:**
   ```rust
   let temp_spin_toml = generate_temp_spin_toml(&GenerateSpinConfig {
       file_system: &deps.file_system,
       project_path: &working_path,
       download_components: true,   // Download registry components
       validate_local_auth: true,   // Validate auth for build
   })?;
   ```

2. **Component Discovery:**
   Parses generated `spin.toml` to find components with build configurations:
   ```toml
   [[component]]
   id = "my-rust-tool"
   source = "components/my-rust-tool/target/wasm32-wasip1/release/my-rust-tool.wasm"
   [component.build]
   command = "cargo build --target wasm32-wasip1 --release"
   workdir = "components/my-rust-tool"
   ```

3. **Parallel Building:**
   Executes build commands for all local components simultaneously

4. **External Component Resolution:**
   Downloads registry components (gateway, authorizer) if not cached

5. **Schema Generation:**
   - Rust: Automatic via `schemars` derive macros
   - Python: Via type hints and `pydantic` 
   - Go: Via struct tags and reflection

6. **Validation:**
   - Verifies WASM modules are valid Component Model format
   - Validates generated JSON schemas for MCP tool descriptions

**Temporary File Lifecycle:**
```
/tmp/ftl-XXXXXX/
└── spin.toml         # Generated, used for discovery, then deleted
```

**Build Output:**
```
my-project/
├── components/
│   └── my-tool/
│       ├── target/wasm32-wasip1/release/my-tool.wasm  # Compiled WASM
│       └── my-tool.schema.json                       # Generated schema
└── .spin/
    └── registry/     # Cached external components
```

### `ftl up`: Local Development Server

**What It Does:**
Starts a local Spin server running your complete MCP application.

**Transpilation Role:**
Generates temporary `spin.toml` with full configuration for local development.

**Behind the Scenes:**

1. **Pre-Build Check:**
   Automatically runs `ftl build` if components need compilation

2. **Development Transpilation:**
   ```rust
   let temp_spin_toml = generate_temp_spin_toml(&GenerateSpinConfig {
       file_system: &deps.file_system,
       project_path: &working_path,  
       download_components: true,   // Ensure all components available
       validate_local_auth: true,   // Validate auth config
   })?;
   ```

3. **Authentication Configuration Expansion:**
   
   **Simple Configuration:**
   ```toml
   # ftl.toml
   access_control = "private"
   ```

   **Generates Complex Auth Setup:**
   ```toml
   # Temporary spin.toml
   [variables]
   auth_enabled = { default = "true" }
   mcp_tenant_id = { required = true }
   mcp_gateway_url = { default = "http://ftl-mcp-gateway.spin.internal/mcp-internal" }
   # ... many more auth variables

   [[component]]
   id = "mcp"  # Authorizer component
   source = { url = "ghcr.io/fastertools/mcp-authorizer:0.0.13" }
   
   [[component]]  
   id = "ftl-mcp-gateway"  # Gateway component
   source = { url = "ghcr.io/fastertools/mcp-gateway:0.0.15" }

   [[trigger.http]]
   route = "/..."
   component = "mcp"  # All requests go through authorizer first
   
   [[trigger.http]]
   route = { private = true }  
   component = "ftl-mcp-gateway"  # Private internal route
   ```

4. **Spin Server Launch:**
   ```bash
   spin up --file /tmp/ftl-XXXXXX/spin.toml --listen 127.0.0.1:3000
   ```

5. **Runtime Architecture:**
   ```
   ┌─────────────────────────────────────┐
   │         Spin Runtime                │
   ├─────────────────────────────────────┤
   │ HTTP Server (0.0.0.0:3000)         │
   │  ├─ /tools/list                     │
   │  ├─ /tools/call                     │
   │  └─ /ping                           │
   ├─────────────────────────────────────┤
   │ Component Network                   │
   │  ├─ mcp (authorizer)                │
   │  ├─ ftl-mcp-gateway                 │
   │  └─ tool components                 │
   └─────────────────────────────────────┘
   ```

6. **Development Features:**
   - File watching for auto-reload
   - Live component logging
   - Detailed error reporting

**Server Output:**
```bash
✅ FTL server started successfully  
🌐 MCP server available at: http://localhost:3000
📋 Available tools:
   - calculator/add_numbers
   - calculator/subtract_numbers  
🔄 Watching for changes...
```

### `ftl deploy`: Production Deployment

**What It Does:**
Deploys your FTL application to production infrastructure.

**Transpilation Role:**
Generates production-optimized temporary `spin.toml` for deployment.

**Behind the Scenes:**

1. **Production Transpilation:**
   ```rust
   let temp_spin_toml = generate_temp_spin_toml(&GenerateSpinConfig {
       file_system: &deps.file_system,
       project_path: &working_path,
       download_components: true,   // Ensure production components
       validate_local_auth: false,  // Production auth validation
   })?;
   ```

2. **Build Verification:**
   ```bash
   ftl build --release  # Ensures optimized WASM builds
   ```

3. **Production Configuration:**
   Generates spin.toml with production settings:
   - Pinned component versions instead of `:latest`
   - Production auth endpoints
   - Optimized resource limits
   - Monitoring and logging configuration

4. **Bundle Preparation:**
   - Creates deployment artifact with all components
   - Includes optimized WASM modules  
   - Packages configuration and dependencies

5. **Infrastructure Provisioning:**
   - Managed infrastructure resources
   - Load balancing and auto-scaling
   - SSL/TLS certificate management

6. **Service Activation:**
   ```
   Upload Pipeline:
   ├── Application Bundle (generated spin.toml + components)
   ├── Environment Configuration  
   ├── Health Check Endpoints
   └── Monitoring Setup
   ```

## File System Reality

### What Actually Exists

**Persistent Files (version controlled):**
```
my-project/
├── ftl.toml          # Your configuration
├── components/       # Tool source code
│   └── my-tool/
│       ├── src/
│       ├── Cargo.toml
│       └── README.md
├── README.md
└── .gitignore
```

**Generated Files (temporary):**
```
/tmp/ftl-XXXXXX/      # Created during command execution
├── spin.toml         # Generated configuration
└── components/       # Downloaded registry components (cached)
```

**Build Artifacts (generated, can be cached):**
```  
my-project/
├── components/
│   └── my-tool/
│       └── target/wasm32-wasip1/release/
│           ├── my-tool.wasm        # Compiled WASM
│           └── my-tool.schema.json # Generated schema
└── .spin/
    └── registry/     # External component cache
```

### What Never Exists

- **Static spin.toml in project directory** - Always generated dynamically
- **Persistent Spin configuration** - Recreated for each command
- **Manual component management** - All handled by transpilation

## Authentication & Variable Expansion

### Simple to Complex Transformation

FTL's transpilation engine dramatically expands simple authentication configuration:

**Your Configuration:**
```toml
[project]  
access_control = "private"
```

**Generated System Variables:**
```toml
[variables]
# Core auth state
auth_enabled = { default = "true" }

# Gateway communication  
mcp_gateway_url = { default = "http://ftl-mcp-gateway.spin.internal/mcp-internal" }
mcp_trace_header = { default = "x-trace-id" }

# Provider configuration
mcp_provider_type = { default = "ftl_authkit" }
mcp_tenant_id = { required = true }  # Platform provides this

# JWT validation
mcp_jwt_issuer = { default = "https://divine-lion-50-staging.authkit.app" }
mcp_jwt_audience = { default = "ftl-mcp-server" }
mcp_jwt_required_scopes = { default = "" }
mcp_jwt_jwks_uri = { default = "" }  # Auto-derived for FTL AuthKit
mcp_jwt_public_key = { default = "" }
mcp_jwt_algorithm = { default = "" }

# OAuth endpoints (empty for FTL AuthKit)
mcp_oauth_authorize_endpoint = { default = "" }  
mcp_oauth_token_endpoint = { default = "" }
mcp_oauth_userinfo_endpoint = { default = "" }

# Legacy support (deprecated)
mcp_static_tokens = { default = "" }
```

### OIDC Configuration Expansion

**Your OIDC Configuration:**
```toml
[project]
access_control = "private"

[oidc]
issuer = "https://auth.company.com"
audience = "my-api"
jwks_uri = "https://auth.company.com/.well-known/jwks.json"
```

**Generated Variables (30+ variables):**
```toml
[variables]
# Provider type changes
mcp_provider_type = { default = "oidc" }

# OIDC-specific values
mcp_jwt_issuer = { default = "https://auth.company.com" }
mcp_jwt_audience = { default = "my-api" }
mcp_jwt_jwks_uri = { default = "https://auth.company.com/.well-known/jwks.json" }

# Plus all the auth_enabled, gateway, and tenant variables...
```

## Debugging & Troubleshooting

### Understanding Transpilation Issues

**Common Problems:**

1. **Invalid ftl.toml Syntax:**
   ```bash
   Error: Failed to parse ftl.toml
   → Check TOML syntax with a validator
   ```

2. **Missing Component Build:**
   ```bash
   Error: Component WASM file not found
   → Run 'ftl build' to compile components
   ```

3. **Auth Configuration Conflicts:**
   ```bash
   Error: Both access_control="public" and [oidc] specified
   → Private access required for OIDC
   ```

### Inspecting Generated Configuration

**Export Generated spin.toml:**
```bash
ftl build --export spin --export-out debug-spin.toml
```

This creates a permanent copy of the generated configuration for inspection.

**Verbose Transpilation:**
```bash
ftl build --verbose
```

Shows detailed transpilation process and variable injection.

### Component Build Debugging

**Individual Component Build:**
```bash
cd components/my-tool
make build  # or cargo build, etc.
```

**Component Discovery:**
```bash
ftl build --verbose  # Shows detailed build information
```

### Runtime Debugging

**Development Logging:**
```bash
ftl up --log-level debug
```

**Component-Specific Logs:**
```bash
spin logs --component my-tool  # During ftl up
```

## Advanced Topics

### Custom Authentication Providers

FTL supports complex authentication scenarios through transpilation:

```toml
[project]
access_control = "private"

[oidc]
issuer = "https://custom-auth.company.com"
audience = "api://my-service"
authorize_endpoint = "https://custom-auth.company.com/oauth2/authorize"
token_endpoint = "https://custom-auth.company.com/oauth2/token"
userinfo_endpoint = "https://custom-auth.company.com/oauth2/userinfo"
jwks_uri = "https://custom-auth.company.com/.well-known/jwks.json"
public_key = "-----BEGIN PUBLIC KEY-----\n..."
algorithm = "RS256"
required_scopes = "read write admin"
```

This generates 20+ authentication variables in the runtime configuration.

### Multi-Environment Configuration

Use different `ftl.toml` files for different environments:

```bash
# Development
ftl up --config ftl.dev.toml

# Staging  
ftl deploy --config ftl.staging.toml

# Production
ftl deploy --config ftl.prod.toml
```

Each generates appropriate runtime configuration for that environment.

### Performance Optimization

**Build Performance:**
- **Component Caching:** Registry components cached in `~/.cache/spin/`
- **Incremental Builds:** Only changed components recompiled
- **Parallel Compilation:** All components built simultaneously

**Runtime Performance:**
- **WASM Startup:** Components start in ~1-5ms
- **Memory Efficiency:** Linear memory model with precise control
- **Internal Communication:** Components communicate via shared memory

## Next Steps

Understanding FTL's dynamic transpilation system helps you:

- **Debug Configuration Issues** - Know where problems originate
- **Optimize Build Performance** - Understand caching and incremental builds  
- **Design Complex Auth** - Leverage variable injection for custom providers
- **Extend FTL** - Build on the transpilation architecture

**Continue Learning:**
- **[Getting Started Tutorials](../getting-started/)** - Apply your knowledge
- **[ftl.toml Reference](../ftl-schema/ftl-toml-reference.md)** - Complete configuration options
- **[SDK Reference](../sdk-reference/)** - Build tool components
- **[Examples](../../examples/)** - Real-world configuration patterns

The transpilation system is FTL's core innovation - it allows simple, maintainable configuration while providing the full power of Spin's component architecture.