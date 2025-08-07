# FTL Configuration Reference (ftl.toml)

`ftl.toml` is the primary configuration file for FTL projects. It defines:
- Project metadata and settings
- Access control
- Tool configurations and build settings
- MCP gateway and authorizer component config
- Variables and environment configuration

## File Structure

```toml
[project]
name = "my-ftl-project"
version = "0.1.0"
description = "My FTL MCP project"
authors = ["Your Name <you@example.com>"]
access_control = "public"  # or "private"
default_registry = "ghcr.io/myorg"  # Optional: default registry for components

[oidc]  # Optional - only needed for custom OIDC providers
issuer = "https://auth.example.com"
audience = "my-api"
# ... additional OIDC settings

[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = false

[variables]
API_KEY = { default = "test-key" }
REQUIRED_VAR = { required = true }

[tools.my-tool]
path = "tools/my-tool"  # Optional, defaults to tool name
wasm = "target/wasm32-wasip1/release/my_tool.wasm"
allowed_outbound_hosts = ["https://api.example.com"]

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml"]
env = { RUST_LOG = "info" }

[tools.my-tool.variables]
TOOL_CONFIG = "value"
```

## Project Section

The `[project]` section contains essential metadata about your FTL project.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | - | Project name (alphanumeric with dashes/underscores) |
| `version` | string | No | "0.1.0" | Project version (SemVer format) |
| `description` | string | No | "" | Project description |
| `authors` | array | No | [] | List of project authors |
| `access_control` | string | No | "public" | Access control mode: "public" or "private" |
| `default_registry` | string | No | - | Default registry for component references (e.g., "ghcr.io/myorg") |

### Access Control Modes

- **`public`** (default): No authentication required. The MCP endpoint is publicly accessible.
- **`private`**: Authentication required. When set to private:
  - Without `[oidc]` section: Uses FTL's built-in auth provider
  - With `[oidc]` section: Uses your custom OIDC provider

#### Example: Using FTL's Built-in Provider

Set `access_control = "private"` without an `[oidc]` section:

```toml
[project]
name = "secure-tools"
version = "1.0.0"
description = "Collection of MCP tools for data processing"
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
access_control = "private"
# No [oidc] section needed - uses FTL's provider automatically. Only the owner of the FTL server is authorized to access it.
```

### Default Registry

The `default_registry` field simplifies component references throughout your configuration. When set, you can use short component names that will be automatically prefixed with the registry URL.

#### Example: Using Default Registry

```toml
[project]
name = "my-project"
default_registry = "ghcr.io/myorg"

[mcp]
# Short references - will resolve to ghcr.io/myorg/...
gateway = "mcp-gateway:1.0.0"
authorizer = "mcp-authorizer:1.0.0"

[tools.my-tool]
# Component from registry using short reference
wasm = "some-component:1.0.0"  # Resolves to ghcr.io/myorg/some-component:1.0.0
allowed_outbound_hosts = ["https://api.example.com"]
# Note: build section is omitted for registry components
```

Without a default registry, you must use full component references:

```toml
[tools.my-tool]
# Full registry reference required
wasm = "ghcr.io/myorg/some-component:1.0.0"
allowed_outbound_hosts = ["https://api.example.com"]
# Note: build section is omitted for registry components
```

## OIDC Section (Optional)

The `[oidc]` section configures custom OpenID Connect authentication. This section is only used when `access_control = "private"`.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `issuer` | string | Yes | - | OIDC issuer URL |
| `audience` | string | No | "" | Expected audience for tokens |
| `jwks_uri` | string | No | "" | JWKS endpoint URL (auto-discovered if not set) |
| `public_key` | string | No | "" | Public key in PEM format (alternative to JWKS) |
| `algorithm` | string | No | "" | JWT signature algorithm (e.g., RS256, ES256) |
| `required_scopes` | string | No | "" | Comma-separated list of required scopes |
| `authorize_endpoint` | string | No | "" | OAuth authorization endpoint |
| `token_endpoint` | string | No | "" | OAuth token endpoint |
| `userinfo_endpoint` | string | No | "" | OAuth userinfo endpoint |

### Example: Using Auth0

```toml
[project]
name = "secure-tools"
access_control = "private"

[oidc]
issuer = "https://your-tenant.auth0.com/"
audience = "https://api.example.com"
jwks_uri = "https://your-tenant.auth0.com/.well-known/jwks.json"
required_scopes = "read:data,write:data"
```

## MCP Section

The `[mcp]` section configures the MCP gateway and authorizer components.

### Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `gateway` | string | No | "ghcr.io/fastertools/mcp-gateway:0.0.11" | MCP gateway component registry URI |
| `authorizer` | string | No | "ghcr.io/fastertools/mcp-authorizer:0.0.13" | MCP authorizer component registry URI |
| `validate_arguments` | boolean | No | false | Whether to validate tool call arguments |

### Component References

Components can be specified as:
- **Full registry URLs**: `ghcr.io/fastertools/mcp-gateway:0.0.11`
- **Short references** (when `default_registry` is set): `mcp-gateway:0.0.11`
- **Local paths**: `target/wasm32-wasip1/release/my_component.wasm`

### Example

```toml
[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = true  # Enable strict argument validation
```

With default registry:

```toml
[project]
default_registry = "ghcr.io/fastertools"

[mcp]
gateway = "mcp-gateway:0.0.11"  # Resolves to ghcr.io/fastertools/mcp-gateway:0.0.11
authorizer = "mcp-authorizer:0.0.13"
```

## Variables Section

The `[variables]` section defines application-level environment variables available to all components.

### Syntax

Variables can be defined in two ways:

1. **With default value**: `VAR_NAME = { default = "value" }`
2. **Required (no default)**: `VAR_NAME = { required = true }`

### Example

```toml
[variables]
API_ENDPOINT = { default = "https://api.example.com" }
LOG_LEVEL = { default = "info" }
SECRET_KEY = { required = true }  # Must be provided at runtime
```

## Tools Sections

Each tool in your project is configured with a `[tools.TOOL_NAME]` section. Tools can be either built locally or pulled from a registry.

### Tool Configuration Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `path` | string | No | Tool name | Path to tool directory (for local tools only) |
| `wasm` | string | Yes | - | Path to WASM file OR registry reference |
| `allowed_outbound_hosts` | array | No | [] | List of allowed external hosts |
| `variables` | table | No | {} | Tool-specific variables |

### Component Sources

The `wasm` field can specify either:
1. **Local file path**: `"target/wasm32-wasip1/release/my_tool.wasm"`
2. **Registry reference**: `"ghcr.io/org/component:1.0.0"`
3. **Short registry reference** (with default_registry): `"component:1.0.0"`

### Build Configuration

The `[tools.TOOL_NAME.build]` subsection configures how the tool is built. **This section is only required for locally-built tools, not for registry components.**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `command` | string | Yes* | - | Build command to execute (*for local tools) |
| `watch` | array | No | [] | Glob patterns for files to watch in dev mode |
| `env` | table | No | {} | Environment variables for build process |

### Example: Rust Tool

```toml
[tools.data-processor]
path = "tools/data-processor"  # Optional if folder name matches tool name
wasm = "target/wasm32-wasip1/release/data_processor.wasm"
allowed_outbound_hosts = [
    "https://api.github.com",
    "https://*.amazonaws.com"
]

[tools.data-processor.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs", "Cargo.toml", "Cargo.lock"]
env = { RUST_LOG = "debug" }

[tools.data-processor.variables]
PROCESSING_MODE = "batch"
MAX_WORKERS = "4"
```

### Example: TypeScript Tool

```toml
[tools.web-scraper]
wasm = "build/web-scraper.wasm"
allowed_outbound_hosts = ["https://*"]  # Allow all HTTPS hosts

[tools.web-scraper.build]
command = "npm run build"
watch = ["src/**/*.ts", "package.json"]
env = { NODE_ENV = "production" }

[tools.web-scraper.variables]
USER_AGENT = "FTL-Scraper/1.0"
```

### Example: Python Tool

```toml
[tools.ml-analyzer]
wasm = "app.wasm"
allowed_outbound_hosts = ["https://huggingface.co"]

[tools.ml-analyzer.build]
command = "spin py2wasm app -o app.wasm"
watch = ["*.py", "requirements.txt"]

[tools.ml-analyzer.variables]
MODEL_NAME = "bert-base-uncased"
BATCH_SIZE = "32"
```

### Example: Registry Component

```toml
# Using a pre-built component from a registry
[tools.json-formatter]
wasm = "ghcr.io/fastertools/ftl-tool-json-formatter:0.0.1"
allowed_outbound_hosts = []  # No external access needed

[tools.json-formatter.variables]
INDENT_SIZE = "2"
# Note: No build section needed for registry components
```

### Example: Mixed Local and Registry Components

```toml
[project]
name = "my-project"
default_registry = "ghcr.io/myorg"

# Local component with build configuration
[tools.custom-processor]
path = "tools/processor"
wasm = "target/wasm32-wasip1/release/processor.wasm"

[tools.custom-processor.build]
command = "cargo build --target wasm32-wasip1 --release"

# Registry component using short reference
[tools.data-validator]
wasm = "validator:2.1.0"  # Resolves to ghcr.io/myorg/validator:2.1.0
allowed_outbound_hosts = ["https://api.validator.com"]
# No build section for registry components
```

## Advanced Features

### Build Profiles

For complex build scenarios, you can define multiple build profiles:

```toml
[tools.my-tool]
wasm = "target/wasm32-wasip1/release/my_tool.wasm"

[tools.my-tool.build]
command = "cargo build --target wasm32-wasip1"  # Default dev build

[tools.my-tool.profiles.release]
command = "cargo build --target wasm32-wasip1 --release"
env = { CARGO_PROFILE_RELEASE_LTO = "true" }

[tools.my-tool.up]
profile = "debug"  # Profile to use for 'ftl up'

[tools.my-tool.deploy]
profile = "release"  # Profile to use for deployment
```

### Variable Templates

Tool variables can reference application variables using template syntax:

```toml
[variables]
BASE_URL = { default = "https://api.example.com" }

[tools.api-client.variables]
ENDPOINT = "{{ BASE_URL }}/v1"  # Will be replaced with BASE_URL value
```

## Complete Example

Here's a complete example of an ftl.toml file for a multi-tool project with authentication:

```toml
# Project metadata
[project]
name = "enterprise-mcp-suite"
version = "2.0.0"
description = "Enterprise MCP tool suite with authentication"
authors = ["DevOps Team <devops@company.com>"]
access_control = "private"

# Custom OIDC authentication
[oidc]
issuer = "https://auth.company.com"
audience = "mcp-suite-api"
jwks_uri = "https://auth.company.com/.well-known/jwks.json"
required_scopes = "mcp:read,mcp:write"

# MCP components configuration
[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.13"
validate_arguments = true

# Application variables
[variables]
LOG_LEVEL = { default = "info" }
API_BASE_URL = { default = "https://api.company.com" }
SECRET_TOKEN = { required = true }

# Database query tool
[tools.db-query]
path = "tools/database"
wasm = "target/wasm32-wasip1/release/db_query.wasm"
allowed_outbound_hosts = ["postgres://db.company.com"]

[tools.db-query.build]
command = "cargo build --target wasm32-wasip1 --release"
watch = ["src/**/*.rs"]

[tools.db-query.variables]
DB_CONNECTION_STRING = "{{ DATABASE_URL }}"
QUERY_TIMEOUT = "30"

# File processor tool
[tools.file-processor]
wasm = "build/processor.wasm"
allowed_outbound_hosts = ["https://storage.company.com"]

[tools.file-processor.build]
command = "npm run build:wasm"
watch = ["src/**/*.js", "package.json"]

[tools.file-processor.variables]
STORAGE_BUCKET = "mcp-files"
MAX_FILE_SIZE = "10485760"  # 10MB
```

## Registry Management

FTL supports pulling components from OCI-compatible registries. Authentication is handled through Docker's credential store.

### Setting up Registry Authentication

Use `docker login` to authenticate with any registry:

```bash
# GitHub Container Registry
docker login ghcr.io

# Docker Hub
docker login docker.io

# AWS ECR
aws ecr get-login-password | docker login --username AWS --password-stdin 123456789.dkr.ecr.us-west-2.amazonaws.com
```

### Registry CLI Commands

FTL provides commands to manage registry configuration:

```bash
# List current registry configuration
ftl registry list

# Set default registry
ftl registry set ghcr.io/myorg

# Remove default registry
ftl registry remove
```

### Using Registry Components

Components from registries are automatically pulled during deployment. Specify them using:

1. **Full registry URLs**: `ghcr.io/org/component:1.0.0`
2. **Short names** (with default_registry): `component:1.0.0`
3. **Latest tags**: Automatically resolved to the latest semantic version

## Migration from spin.toml

If you have an existing `spin.toml` file, you can migrate to `ftl.toml` by:

1. Extract project metadata to the `[project]` section
2. Move authentication configuration to `access_control` and optionally `[oidc]`
3. Convert Spin components to `[tools.*]` sections
4. Update variable definitions to use the FTL format

## Export to spin.toml

Run `ftl build --export spin` in any `ftl` project to generate a `spin.toml` from your `ftl.toml`.

## Validation

FTL validates your configuration when running commands. Common validation rules include:

- Project name must be alphanumeric with dashes/underscores
- Tool names must be valid identifiers
- `access_control` must be either "public" or "private"
- Required fields must be present
- WASM paths must be specified for each tool
