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
path = "my-tool"             # Required: Path to tool directory
allowed_outbound_hosts = [   # Optional: Allowed outbound hosts
    "https://api.example.com"
]

[tools.my-tool.build]        # Required: Build configuration
command = "cargo build --target wasm32-wasip1 --release"  # Build command
workdir = "."                # Optional: Working directory (relative to tool path)
watch = [                    # Optional: Paths to watch for changes
    "src/**/*.rs",
    "Cargo.toml"
]
env = { RUSTFLAGS = "-C opt-level=z" }  # Optional: Environment variables
```

The build configuration is always explicit, giving you full visibility and control:
- **command**: The exact build command to run
- **workdir**: Working directory for the build (defaults to the tool path)
- **watch**: File patterns to watch in development mode
- **env**: Environment variables to set during the build

When you run `ftl add`, it will create the appropriate build configuration based on your chosen language:
- **Rust**: Creates a cargo build configuration with wasm32-wasip1 target
- **TypeScript/JavaScript**: Creates an npm build configuration

### Authentication Section

```toml
[auth]
enabled = false              # Required: Enable/disable authentication
provider = "authkit"         # Required if enabled: Auth provider type
issuer = "https://..."       # Required if enabled: OIDC issuer URL
audience = "mcp-api"         # Required if enabled: Expected audience

# Optional: OIDC-specific configuration
[auth.oidc]
provider_name = "My Auth"
jwks_uri = "https://..."
authorize_endpoint = "https://..."
token_endpoint = "https://..."
userinfo_endpoint = "https://..."
allowed_domains = "example.com,other.com"
```

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
provider = "authkit"
issuer = "https://my-tenant.authkit.app"
audience = "weather-api"

[tools.weather]
type = "typescript"
path = "weather-tool"
allowed_outbound_hosts = ["https://api.openweathermap.org"]
watch = ["src/**/*.ts", "package.json"]

[tools.forecast]
type = "rust"
path = "forecast-tool"
build = "cargo build --release"

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