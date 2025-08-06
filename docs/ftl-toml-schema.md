# FTL Configuration Schema (ftl.toml)

This document provides detailed schema information for the `ftl.toml` configuration file format.

## Schema Overview

The FTL configuration uses TOML format with the following top-level sections:

- `[project]` - **Required**: Project metadata and settings
- `[oidc]` - **Optional**: OpenID Connect authentication configuration  
- `[mcp]` - **Optional**: MCP gateway and authorizer settings
- `[variables]` - **Optional**: Application-level variables
- `[tools.*]` - **Optional**: Tool definitions and configurations

## Type Definitions

### ProjectConfig

The main project configuration section.

```toml
[project]
name = "string"           # Required: Project name
version = "string"        # Optional: Version (default: "0.1.0")
description = "string"    # Optional: Description
authors = ["string", ...] # Optional: List of authors
access_control = "string" # Optional: "public" or "private" (default: "public")
```

**Validation Rules:**
- `name`: Must match pattern `^[a-zA-Z][a-zA-Z0-9_-]*$`
- `name`: Minimum length 1
- `version`: Minimum length 1 if provided
- `access_control`: Must be either "public" or "private"

### OidcConfig

OpenID Connect authentication configuration (used when `access_control = "private"`).

```toml
[oidc]
issuer = "string"              # Required: OIDC issuer URL
audience = "string"            # Optional: Expected audience
jwks_uri = "string"            # Optional: JWKS endpoint URL
public_key = "string"          # Optional: Public key in PEM format
algorithm = "string"           # Optional: JWT algorithm (RS256, ES256, etc.)
required_scopes = "string"     # Optional: Comma-separated required scopes
authorize_endpoint = "string"  # Optional: OAuth authorization endpoint
token_endpoint = "string"      # Optional: OAuth token endpoint
userinfo_endpoint = "string"   # Optional: OAuth userinfo endpoint
```

**Validation Rules:**
- `issuer`: Minimum length 1
- Either `jwks_uri` or `public_key` should be provided (not both)

### McpConfig

MCP component configuration.

```toml
[mcp]
gateway = "string"              # Optional: Gateway registry URI
authorizer = "string"           # Optional: Authorizer registry URI
validate_arguments = boolean   # Optional: Argument validation flag
```

**Default Values:**
- `gateway`: "ghcr.io/fastertools/mcp-gateway:0.0.10"
- `authorizer`: "ghcr.io/fastertools/mcp-authorizer:0.0.12"
- `validate_arguments`: false

### ApplicationVariable

Variables can be defined with either a default value or marked as required.

```toml
[variables]
# Format 1: Variable with default value
VAR_NAME = { default = "string" }

# Format 2: Required variable
VAR_NAME = { required = true }
```

### ToolConfig

Tool configuration with build settings and variables.

```toml
[tools.TOOL_NAME]
path = "string"                        # Optional: Path to tool directory
wasm = "string"                        # Required: Path to WASM file
allowed_outbound_hosts = ["string", ...] # Optional: Allowed external hosts
variables = { KEY = "value", ... }    # Optional: Tool-specific variables

[tools.TOOL_NAME.build]
command = "string"                     # Required: Build command
watch = ["string", ...]                # Optional: Watch patterns
env = { KEY = "value", ... }          # Optional: Build environment variables

[tools.TOOL_NAME.up]                  # Optional: Development configuration
profile = "string"                     # Required if section exists: Build profile

[tools.TOOL_NAME.deploy]               # Optional: Deployment configuration
profile = "string"                     # Required if section exists: Build profile
name = "string"                        # Optional: Custom deployment name suffix

[tools.TOOL_NAME.profiles.PROFILE_NAME] # Optional: Named build profiles
command = "string"                     # Required: Build command
watch = ["string", ...]                # Optional: Watch patterns
env = { KEY = "value", ... }          # Optional: Environment variables
```

**Validation Rules:**
- `wasm`: Minimum length 1
- `build.command`: Minimum length 1
- Tool names must contain only alphanumeric characters, dashes, or underscores

## Authentication Modes

### Public Mode (Default)

No authentication required:

```toml
[project]
name = "my-project"
access_control = "public"  # or omit entirely
```

### Private Mode with FTL AuthKit

Uses FTL's built-in authentication:

```toml
[project]
name = "my-project"
access_control = "private"
# No [oidc] section needed
```

### Private Mode with Custom OIDC

Uses your own OIDC provider:

```toml
[project]
name = "my-project"
access_control = "private"

[oidc]
issuer = "https://auth.example.com"
audience = "my-api"
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
```

## Variable Resolution

Variables are resolved in the following order:

1. **Tool Variables**: Defined in `[tools.TOOL_NAME.variables]`
2. **Application Variables**: Defined in `[variables]`
3. **Environment Variables**: Passed at runtime
4. **Template References**: Variables can reference other variables using `{{ VAR_NAME }}`

Example:

```toml
[variables]
API_BASE = { default = "https://api.example.com" }

[tools.my-tool.variables]
ENDPOINT = "{{ API_BASE }}/v1"  # Resolved to https://api.example.com/v1
```

## Build Profiles

Tools can define multiple build profiles for different scenarios:

```toml
[tools.my-tool]
wasm = "target/release/tool.wasm"

# Default build configuration
[tools.my-tool.build]
command = "cargo build"

# Named profiles
[tools.my-tool.profiles.debug]
command = "cargo build"
env = { RUST_LOG = "debug" }

[tools.my-tool.profiles.release]
command = "cargo build --release"
env = { CARGO_PROFILE_RELEASE_LTO = "true" }

# Profile selection
[tools.my-tool.up]
profile = "debug"  # Use debug profile for development

[tools.my-tool.deploy]
profile = "release"  # Use release profile for deployment
```

## Allowed Outbound Hosts

Tools can specify which external hosts they're allowed to connect to:

```toml
[tools.web-client]
allowed_outbound_hosts = [
    "https://api.example.com",      # Specific host
    "https://*.example.com",        # Wildcard subdomain
    "https://*",                    # All HTTPS hosts
    "postgres://db.example.com",    # Database connection
    "redis://cache.example.com:6379" # Redis with port
]
```

## Complete Schema Example

```toml
# Project configuration
[project]
name = "complete-example"
version = "1.0.0"
description = "Complete ftl.toml example"
authors = ["Dev Team <dev@example.com>"]
access_control = "private"

# OIDC authentication
[oidc]
issuer = "https://auth.example.com"
audience = "api.example.com"
jwks_uri = "https://auth.example.com/.well-known/jwks.json"
algorithm = "RS256"
required_scopes = "read,write"
authorize_endpoint = "https://auth.example.com/authorize"
token_endpoint = "https://auth.example.com/token"
userinfo_endpoint = "https://auth.example.com/userinfo"

# MCP components
[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = true

# Application variables
[variables]
LOG_LEVEL = { default = "info" }
API_KEY = { required = true }
BASE_URL = { default = "https://api.example.com" }

# Tool with all features
[tools.advanced-tool]
path = "tools/advanced"
wasm = "target/wasm32-wasip1/release/tool.wasm"
allowed_outbound_hosts = ["https://*"]

[tools.advanced-tool.build]
command = "cargo build --target wasm32-wasip1"
watch = ["src/**/*.rs", "Cargo.toml"]
env = { RUST_LOG = "info" }

[tools.advanced-tool.profiles.debug]
command = "cargo build --target wasm32-wasip1"
env = { RUST_LOG = "debug", RUST_BACKTRACE = "1" }

[tools.advanced-tool.profiles.release]
command = "cargo build --target wasm32-wasip1 --release"
env = { CARGO_PROFILE_RELEASE_LTO = "true" }

[tools.advanced-tool.up]
profile = "debug"

[tools.advanced-tool.deploy]
profile = "release"
name = "advanced-prod"

[tools.advanced-tool.variables]
TOOL_MODE = "production"
API_ENDPOINT = "{{ BASE_URL }}/tools"
MAX_RETRIES = "3"
```

## Validation Error Messages

Common validation errors and their meanings:

| Error | Meaning |
|-------|---------|
| `project.name: length is lower than 1` | Project name is required |
| `project.name: does not match pattern` | Project name contains invalid characters |
| `project.access_control: Invalid access_control` | Must be "public" or "private" |
| `tools.*.build.command: length is lower than 1` | Build command is required |
| `Tool name contains invalid characters` | Tool names must be alphanumeric with dashes/underscores |
| `oidc.issuer: length is lower than 1` | OIDC issuer is required when [oidc] section exists |

## Migration Guide

### From Old Auth Structure

**Old format (deprecated):**
```toml
[auth]
enabled = true
provider = "oidc"

[auth.oidc]
issuer = "https://auth.example.com"
audience = "my-api"
```

**New format:**
```toml
[project]
access_control = "private"

[oidc]
issuer = "https://auth.example.com"
audience = "my-api"
```

### From spin.toml

**Spin format:**
```toml
[component.my-tool]
source = { url = "file://./tool.wasm" }
allowed_outbound_hosts = ["https://*"]

[component.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
```

**FTL format:**
```toml
[tools.my-tool]
wasm = "tool.wasm"
allowed_outbound_hosts = ["https://*"]

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
```

## JSON Schema

A JSON Schema file is available at `ftl-schema.json` for IDE validation and autocompletion. Configure your editor to use it:

### VS Code

Add to `.vscode/settings.json`:

```json
{
  "yaml.schemas": {
    "./docs/ftl-schema.json": "ftl.toml"
  }
}
```

### Other Editors

Most modern editors support JSON Schema validation. Refer to your editor's documentation for configuration instructions.

## See Also

- [FTL Configuration Reference](ftl-toml-reference.md) - Complete configuration reference
- [Secrets Guide](SECRETS-GUIDE.md) - Managing secrets and sensitive variables
- [Secret Lifecycle](SECRET-LIFECYCLE.md) - How secrets flow through the system