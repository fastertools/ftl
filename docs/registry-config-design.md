# Registry Configuration Design

## Overview

The registry configuration system will allow users to manage multiple container registries for FTL tools, with support for custom registries, authentication, and default preferences.

## Configuration File Location

- **Primary**: `~/.ftl/registries.toml` (user-specific)
- **Secondary**: `.ftl/registries.toml` (project-specific, overrides user config)
- **Format**: TOML for consistency with spin.toml

## Configuration Schema

```toml
# ~/.ftl/registries.toml

version = "1"
default_registry = "ghcr"  # Which registry to use by default

[[registries]]
name = "ghcr"
type = "ghcr"
enabled = true
priority = 1  # Lower number = higher priority for searches
[registries.config]
organization = "fastertools"

[[registries]]
name = "docker"
type = "docker"
enabled = true
priority = 2

[[registries]]
name = "my-ecr"
type = "ecr"
enabled = true
priority = 3
[registries.config]
account_id = "123456789012"
region = "us-west-2"

[[registries]]
name = "my-custom"
type = "custom"
enabled = true
priority = 4
[registries.config]
url_pattern = "registry.company.com/{image_name}:latest"
auth_type = "basic"  # or "bearer", "none"
```

## Command Structure

### `ftl registries` - Registry management commands

```bash
# List configured registries
ftl registries list

# Add a new registry
ftl registries add ghcr --type ghcr --org fastertools
ftl registries add docker --type docker
ftl registries add my-ecr --type ecr --account 123456789012 --region us-west-2
ftl registries add custom --type custom --url-pattern "registry.company.com/{image_name}:latest"

# Remove a registry
ftl registries remove my-ecr

# Set default registry
ftl registries set-default docker

# Enable/disable a registry
ftl registries enable docker
ftl registries disable my-ecr

# Set registry priority (for search order)
ftl registries set-priority docker 1
```

## Integration with Existing Commands

### Updated `ftl tools list`
```bash
# List from all enabled registries (default)
ftl tools list

# List from specific registry
ftl tools list --registry docker

# List from multiple registries
ftl tools list --registry ghcr,docker
```

### Updated `ftl tools add`
```bash
# Add from default registry
ftl tools add divide

# Add from specific registry
ftl tools add divide --registry docker

# Add with registry prefix (overrides default)
ftl tools add docker:divide
ftl tools add ghcr:divide
```

## Implementation Plan

1. **Config Module** (`src/config/mod.rs`)
   - Registry configuration types
   - Loading/saving logic
   - Default configurations

2. **Update Registry Module** (`src/registry.rs`)
   - Add custom registry adapter
   - Registry manager for handling multiple registries
   - Priority-based registry selection

3. **Registries Command** (`src/commands/registries.rs`)
   - Subcommands for registry management
   - Configuration validation

4. **Update Tools Command**
   - Multi-registry search support
   - Registry prefix parsing
   - Aggregated results handling

## Migration Strategy

1. **Backward Compatibility**
   - If no config exists, create default with ghcr, docker, ecr
   - `--registry` flag continues to work as before
   - Old behavior maintained if config doesn't exist

2. **First Run Experience**
   ```
   $ ftl tools list
   → No registry configuration found. Creating default configuration...
   ✓ Created ~/.ftl/registries.toml with default registries (ghcr, docker, ecr)
   → Use 'ftl registries list' to manage your registries
   ```

## Authentication Considerations

- **Keyring Integration**: Store registry credentials securely using existing keyring approach
- **Environment Variables**: Support standard Docker/AWS env vars
- **Auth Commands**: `ftl registries auth <name>` for interactive authentication

## Error Handling

- Clear messages when registry is unreachable
- Fallback to other registries if one fails
- Warning when no registries are enabled
- Validation of custom URL patterns