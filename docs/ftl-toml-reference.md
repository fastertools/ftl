# ftl.toml Configuration Reference

The `ftl.toml` file is the main configuration file for FTL projects. It defines your project metadata, authentication settings, gateway components, tools, and variables.

## Table of Contents

- [Project Section](#project-section)
- [MCP Section](#mcp-section)
- [Authentication Section](#authentication-section)
- [Variables Section](#variables-section)
- [Tools Section](#tools-section)

## Project Section

The `[project]` section contains basic metadata about your FTL project.

```toml
[project]
name = "my-ftl-project"
version = "0.1.0"
description = "My FTL MCP server"
authors = ["Your Name <you@example.com>"]
```

## MCP Section

The `[mcp]` section configures the Model Context Protocol gateway and authorizer components. You can use the default FTL components or specify custom implementations.

When authentication is enabled, the authorizer component handles all incoming requests at the wildcard route `/...`, validating tokens before forwarding to the internal gateway. When authentication is disabled, all requests go directly to the gateway.

```toml
[mcp]
# Full registry URIs for MCP components
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.10"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.12"
validate_arguments = true
```

### Custom MCP Components

You can use your own MCP implementations by specifying different registry URIs:

```toml
[mcp]
# Use custom implementations
gateway = "ghcr.io/myorg/custom-mcp-gateway:1.0.0"
authorizer = "ghcr.io/myorg/custom-mcp-authorizer:1.0.0"
validate_arguments = true
```

This allows you to:
- Fork and modify the official components
- Create entirely custom implementations
- Use components from different registries (Docker Hub, GHCR, ECR, etc.)
- Maintain full control over your gateway infrastructure

## Authentication Section

The `[auth]` section configures authentication for your MCP server.

### Disabling Authentication

```toml
[auth]
enabled = false
```

### AuthKit Configuration

```toml
[auth]
enabled = true

[auth.authkit]
issuer = "https://your-tenant.authkit.app"
audience = "mcp-api"  # optional
required_scopes = "mcp:read,mcp:write"  # optional - comma-separated list of required scopes
```

### OIDC Configuration

```toml
[auth]
enabled = true

[auth.oidc]
issuer = "https://your-domain.auth0.com"
audience = "your-api-identifier"  # optional
jwks_uri = "https://your-domain.auth0.com/.well-known/jwks.json"
public_key = ""  # optional - PEM format public key (alternative to JWKS)
algorithm = "RS256"  # optional - JWT signing algorithm
required_scopes = "read,write"  # optional - comma-separated list of required scopes
authorize_endpoint = "https://your-domain.auth0.com/authorize"  # optional - for OAuth discovery
token_endpoint = "https://your-domain.auth0.com/oauth/token"  # optional - for OAuth discovery
userinfo_endpoint = "https://your-domain.auth0.com/userinfo"  # optional - for OAuth discovery
```

### Static Token Configuration (Development Only)

For development and testing, you can use static tokens:

```toml
[auth]
enabled = true

[auth.static_token]
tokens = "dev-token:client1:user1:read,write;admin-token:admin:admin:admin:1735689600"
required_scopes = "read"  # optional - comma-separated list of required scopes
```

Token format: `token:client_id:sub:scope1,scope2[:expires_at]`
- Multiple tokens separated by semicolons
- Expiration timestamp is optional (Unix timestamp)

## Variables Section

The `[variables]` section defines application-level variables that can be used by your tools.

### Variables with Default Values

```toml
[variables]
api_url = { default = "https://api.example.com" }
environment = { default = "production" }
```

### Required Variables

```toml
[variables]
api_token = { required = true }
database_url = { required = true }
```

## Tools Section

Each tool in your project gets its own `[tools.<name>]` section.

### Basic Tool Configuration

```toml
[tools.my-tool]
path = "my-tool"
wasm = "my-tool/target/wasm32-wasip1/release/my_tool.wasm"
allowed_outbound_hosts = ["https://api.example.com"]

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]
env = { RUST_LOG = "debug" }
```

### Tool Variables

Tools can access application variables using template syntax:

```toml
[tools.my-tool]
variables = { 
    api_token = "{{ api_token }}", 
    api_url = "{{ api_url }}" 
}
```

### Deploy Configuration

Configure how tools are deployed:

```toml
[tools.my-tool.deploy]
name = "custom-deployed-name"  # Override the deployed component name
profile = "release"  # or "debug"
```

### Build Profiles

Define multiple build profiles:

```toml
[tools.my-tool.build.profiles.dev]
command = "cargo build --target wasm32-wasip1"
watch = ["src/**/*.rs"]

[tools.my-tool.build.profiles.release]
command = "cargo build --target wasm32-wasip1 --release"
```

### Development Mode Configuration

Configure which profile to use for `ftl up`:

```toml
[tools.my-tool.up]
profile = "dev"
```

## Complete Example

```toml
[project]
name = "my-mcp-server"
version = "0.1.0"
description = "My custom MCP server with multiple tools"
authors = ["Jane Doe <jane@example.com>"]

[mcp]
# Using custom MCP components
gateway = "ghcr.io/myorg/enhanced-mcp-gateway:2.0.0"
authorizer = "ghcr.io/myorg/enhanced-mcp-authorizer:2.0.0"
validate_arguments = true

[auth]
enabled = true

[auth.oidc]
issuer = "https://auth.mycompany.com"
audience = "mcp-api"
jwks_uri = "https://auth.mycompany.com/.well-known/jwks.json"
required_scopes = "mcp:read,mcp:write"
authorize_endpoint = "https://auth.mycompany.com/authorize"
token_endpoint = "https://auth.mycompany.com/oauth/token"

[variables]
api_key = { required = true }
api_url = { default = "https://api.mycompany.com" }
environment = { default = "production" }

[tools.data-processor]
path = "tools/data-processor"
wasm = "tools/data-processor/target/wasm32-wasip1/release/data_processor.wasm"
allowed_outbound_hosts = ["https://api.mycompany.com", "https://*.s3.amazonaws.com"]
variables = { api_key = "{{ api_key }}", api_url = "{{ api_url }}" }

[tools.data-processor.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]

[tools.data-processor.deploy]
name = "data-processor-v2"
profile = "release"

[tools.assistant]
path = "tools/assistant"
wasm = "tools/assistant/dist/assistant.wasm"
allowed_outbound_hosts = ["https://api.openai.com"]

[tools.assistant.build]
command = "npm install && npm run build"
watch = ["src/**/*.ts", "package.json"]
```