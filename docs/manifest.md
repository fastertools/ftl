# The ftl.toml Manifest

The `ftl.toml` file is the manifest for your MCP component. It contains metadata about your component.

## Example

```toml
[component]
name = "my-component"
version = "0.1.0"
description = "An MCP component for AI agents"
```

## `[component]`

The `[component]` section contains metadata about your component.

- `name`: The name of your component. This should be lowercase with hyphens (e.g., `my-component`).
- `version`: The version of your component. This should follow the [SemVer](https://semver.org/) specification.
- `description`: A short description of what your component does.

## Build Configuration

Build configuration is handled through:
- `Makefile` - For custom build commands
- Language-specific build tools (cargo, npm, etc.)

## Runtime Configuration

Runtime configuration such as allowed hosts is configured in `spin.toml` rather than `ftl.toml`.