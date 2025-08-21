# FTL CLI Command Reference

## Overview

The FTL CLI provides comprehensive tooling for building, testing, deploying, and managing MCP servers on WebAssembly.

## Commands

### Core Commands

#### `ftl init`
Initialize a new FTL project with scaffolding and configuration.

```bash
ftl init my-project
cd my-project
```

#### `ftl add`
Add a new tool component to your project.

```bash
ftl add my-tool --language rust
ftl add data-processor --language python
```

#### `ftl build`
Build all components in your project to WebAssembly.

```bash
ftl build
ftl build --release  # Optimized build
```

#### `ftl test`
Run tests for all components.

```bash
ftl test
ftl test --component my-tool  # Test specific component
```

#### `ftl up`
Start a local development server with hot reload.

```bash
ftl up
ftl up --watch  # Auto-rebuild on file changes
ftl up --port 8080  # Custom port
```

### Deployment Commands

#### `ftl deploy`
Deploy your application to FTL Engine or other platforms.

```bash
ftl deploy
ftl deploy --environment production
ftl deploy --dry-run  # Validate without deploying
```

Options:
- `--access-control` - Set access mode (public, private, org, custom)
- `--jwt-issuer` - JWT issuer URL for authentication
- `--jwt-audience` - JWT audience for authentication
- `--var KEY=VALUE` - Set deployment variables

#### `ftl logs`
View application logs from deployed instances.

```bash
# Get logs by app name
ftl logs my-app

# Get logs by app ID
ftl logs 123e4567-e89b-12d3-a456-426614174000

# Get logs from the last hour
ftl logs my-app --since 1h

# Get the last 500 lines
ftl logs my-app --tail 500

# Combine options
ftl logs my-app --since 30m --tail 50
```

Options:
- `--since` - Time range for logs (e.g., '30m', '1h', '7d', RFC3339, or Unix timestamp)
- `--tail` - Number of log lines from the end (1-1000, default: 100)

#### `ftl status`
Check the status of deployed applications.

```bash
ftl status
ftl status my-app
```

#### `ftl delete`
Delete a deployed application.

```bash
ftl delete my-app
ftl delete 123e4567-e89b-12d3-a456-426614174000
```

### Authentication Commands

#### `ftl auth login`
Authenticate with FTL Engine.

```bash
ftl auth login
ftl auth login --machine  # Machine-to-machine auth
```

#### `ftl auth logout`
Clear stored credentials.

```bash
ftl auth logout
```

#### `ftl auth status`
Check authentication status.

```bash
ftl auth status
```

### Organization Commands

#### `ftl org list`
List your organizations.

```bash
ftl org list
```

#### `ftl org select`
Select an active organization.

```bash
ftl org select my-org
```

### Utility Commands

#### `ftl list`
List deployed applications.

```bash
ftl list
ftl list --all  # Include deleted apps
```

#### `ftl synth`
Synthesize a Spin manifest from FTL configuration.

```bash
ftl synth  # Auto-detect ftl.yaml/ftl.json/app.cue
ftl synth -f custom-config.yaml
```

#### `ftl registry`
Manage component registry operations.

```bash
ftl registry push my-component
ftl registry pull namespace:component
```

#### `ftl component`
Manage project components.

```bash
ftl component list
ftl component add new-tool --language go
```

## Global Flags

These flags are available for all commands:

- `--config FILE` - Specify configuration file (default: ./ftl.yaml)
- `--verbose, -v` - Enable verbose output
- `--no-color` - Disable colored output
- `--help, -h` - Show help for any command

## Environment Variables

- `FTL_API_URL` - Override default API endpoint
- `FTL_AUTH_TOKEN` - Provide authentication token
- `FTL_ORG_ID` - Set default organization ID
- `NO_COLOR` - Disable colored output globally

## Configuration Files

FTL supports multiple configuration formats:
- `ftl.yaml` / `ftl.yml` - YAML configuration
- `ftl.json` - JSON configuration  
- `app.cue` - CUE language configuration

## Examples

### Complete Workflow

```bash
# Initialize project
ftl init my-mcp-server
cd my-mcp-server

# Add components
ftl add weather-tool --language python
ftl add math-tool --language rust

# Develop locally
ftl up --watch

# Test
ftl test

# Deploy to production
ftl auth login
ftl deploy --environment production

# Monitor
ftl logs my-mcp-server --since 1h
ftl status my-mcp-server

# Clean up
ftl delete my-mcp-server
```

### Debugging Failed Deployments

```bash
# Check recent logs
ftl logs problematic-app --tail 500

# Check app status
ftl status problematic-app

# Try dry run
ftl deploy --dry-run

# Deploy with verbose output
ftl deploy --verbose
```

## See Also

- [Getting Started Guide](./getting-started/README.md)
- [Architecture Documentation](./architecture.md)
- [SDK References](../sdk/README.md)