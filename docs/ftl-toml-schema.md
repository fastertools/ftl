# FTL Configuration Schema (ftl.toml)

## Overview

The `ftl.toml` file is the primary configuration file for FTL projects. It defines your project metadata, tools, authentication settings, and deployment configuration.

## Schema

### Root Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `project` | object | Yes | Project metadata |
| `auth` | object | No | Authentication configuration |
| `tools` | object | No | Tool definitions |
| `deployment` | object | No | Deployment settings |
| `gateway` | object | No | Gateway component configuration |

### Project Section

```toml
[project]
name = "my-mcp-server"       # Required: Project name (alphanumeric, hyphens, underscores)
version = "0.1.0"            # Optional: Version (defaults to "0.1.0")
description = "My MCP server" # Optional: Project description
authors = ["Name <email>"]   # Optional: List of authors
```

### Tools Section

Define your MCP tools with explicit build configurations:

```toml
[tools.my-tool]
path = "my-tool"             # Optional: Path to tool directory (defaults to tool name)
wasm = "my-tool/target/wasm32-wasip1/release/my_tool.wasm"  # Required: Path to WASM output
allowed_outbound_hosts = [   # Optional: Allowed outbound hosts
    "https://api.example.com"
]
variables = {                # Optional: Tool-specific variables
    "API_KEY" = "{{ api_key }}"
}

[tools.my-tool.build]        # Required: Build configuration
command = "cargo build --target wasm32-wasip1 --release"  # Build command
watch = [                    # Optional: Paths to watch for changes
    "src/**/*.rs",
    "Cargo.toml"
]
env = { RUSTFLAGS = "-C opt-level=z" }  # Optional: Environment variables

[tools.my-tool.deploy]       # Optional: Deployment configuration
profile = "release"          # Build profile to use for deployment
name = "custom-name"         # Optional: Custom name suffix (full name: project-custom-name)
```

The configuration provides full control over your tools:
- **path**: Tool directory (defaults to the tool name if not specified)
- **wasm**: Path to the WebAssembly file produced by the build (required)
- **build.command**: The exact build command to run
- **build.watch**: File patterns to watch in development mode
- **build.env**: Environment variables to set during the build
- **profiles**: Optional build profiles for different environments
- **up.profile**: Build profile to use for `ftl up`
- **deploy.profile**: Build profile to use when deploying
- **deploy.name**: Custom name for the deployed tool

### Build Profiles

For advanced use cases, you can define multiple build profiles:

```toml
[tools.my-tool.profiles.dev]
command = "cargo build --target wasm32-wasip1"
watch = ["src/**/*.rs", "Cargo.toml"]
env = { RUST_LOG = "debug" }

[tools.my-tool.profiles.release]
command = "cargo build --target wasm32-wasip1 --release"
env = { RUST_LOG = "warn" }

[tools.my-tool.profiles.production]
command = "cargo build --target wasm32-wasip1 --release"
env = { RUST_LOG = "error", RUST_BACKTRACE = "1" }

[tools.my-tool.up]
profile = "dev"  # Use dev profile for ftl up

[tools.my-tool.deploy]
profile = "production"  # Use production profile for deployment
```

When you run `ftl add`, it will create the appropriate build configuration based on your chosen language:
- **Rust**: Creates a cargo build configuration with wasm32-wasip1 target
- **TypeScript/JavaScript**: Creates an npm build configuration

### Authentication Section

Authentication is configured with provider-specific subsections:

```toml
[auth]
enabled = false              # Required: Enable/disable authentication

# Option 1: AuthKit configuration
[auth.authkit]
issuer = "https://my-tenant.authkit.app"  # Required: AuthKit issuer URL
audience = "mcp-api"                      # Optional: API audience

# Option 2: OIDC configuration (mutually exclusive with authkit)
[auth.oidc]
issuer = "https://auth.example.com"       # Required: OIDC issuer URL
audience = "api"                          # Optional: API audience  
provider_name = "okta"                    # Required: Provider name
jwks_uri = "https://.../.well-known/jwks.json"  # Required: JWKS endpoint
authorize_endpoint = "https://.../authorize"    # Required: Auth endpoint
token_endpoint = "https://.../token"            # Required: Token endpoint
userinfo_endpoint = "https://.../userinfo"      # Optional: User info endpoint
allowed_domains = "example.com,other.com"       # Optional: Allowed email domains
```

**Note**: You must configure either `authkit` or `oidc`, but not both.

### Deployment Section

```toml
[deployment]
registry = "ghcr.io"         # Optional: Container registry
package = "myorg/my-server"  # Optional: Package name
tag = "latest"               # Optional: Tag/version
```

### Gateway Section

```toml
[gateway]
version = "latest"           # Optional: Gateway version (defaults to latest)
authorizer_version = "latest" # Optional: Authorizer version
validate_arguments = true    # Optional: Validate tool arguments (default: true)
```

## Complete Example

```toml
[project]
name = "weather-mcp-server"
version = "1.0.0"
description = "MCP server providing weather tools"
authors = ["Jane Doe <jane@example.com>"]

[auth]
enabled = true

[auth.authkit]
issuer = "https://my-tenant.authkit.app"
audience = "weather-api"

[tools.weather]
path = "weather-tool"  # Optional, defaults to "weather"
wasm = "weather-tool/dist/weather.wasm"
allowed_outbound_hosts = ["https://api.openweathermap.org"]

[tools.weather.build]
command = "npm install && npm run build"
watch = ["src/**/*.ts", "package.json"]

[tools.weather.deploy]
profile = "release"
name = "weather-api"  # Optional custom name

[tools.forecast]
wasm = "forecast/target/wasm32-wasip1/release/forecast.wasm"
# path defaults to "forecast"

[tools.forecast.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]

[deployment]
registry = "ghcr.io"
package = "myorg/weather-mcp"
tag = "v1.0.0"

[gateway]
validate_arguments = true
```

## Validation Rules

1. **Project Name**: Must contain only alphanumeric characters, hyphens, and underscores
2. **Tool Names**: Must start with a letter and contain only alphanumeric characters, hyphens, and underscores
3. **Tool Types**: Must be one of: `rust`, `typescript`, `javascript`
4. **Paths**: Cannot be empty
5. **Auth**: When enabled, provider, issuer, and audience are required
6. **Versions**: Empty versions default to "latest" for gateway components

## Migration from spin.toml

If you have an existing `spin.toml`, you can migrate to `ftl.toml` by:

1. Extract project metadata to `[project]` section
2. Move tool components to `[tools]` section
3. Configure authentication in `[auth]` section
4. Remove Spin-specific configuration (triggers, variables, etc.)

The FTL CLI will automatically generate the appropriate `spin.toml` when needed.