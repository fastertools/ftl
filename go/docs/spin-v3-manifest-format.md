# Spin v3 Manifest Format

## Key Changes from v2 to v3

1. **Components use dotted notation**: `[component.my-component]` instead of `[[component]]`
2. **Triggers are separate**: `[[trigger.http]]` with `component` reference
3. **No inline triggers**: Triggers are not nested within components

## Example Spin v3 Manifest

```toml
spin_manifest_version = 2

[application]
name = "my-app"
version = "0.1.0"
description = "My WebAssembly application"

# Component definition using dotted notation
[component.weather-service]
source = "weather.wasm"
description = "Weather service component"
allowed_outbound_hosts = ["https://api.weather.com"]

[component.weather-service.build]
command = "cargo build --target wasm32-wasi --release"
workdir = "weather/"

[component.auth-handler]
source = "auth.wasm"
description = "Authentication handler"

# Triggers are defined separately
[[trigger.http]]
component = "weather-service"
route = "/weather/*"
id = "weather-trigger"

[[trigger.http]]
component = "auth-handler"
route = "/auth/*"
id = "auth-trigger"

[[trigger.redis]]
component = "cache-handler"
channel = "cache-updates"
id = "redis-trigger"
```

## Component Properties

- `source`: Path to WASM file (required)
- `description`: Component description
- `allowed_outbound_hosts`: Array of allowed hosts
- `environment`: Environment variables
- `build`: Build configuration
- `key_value_stores`: KV store bindings
- `sqlite_databases`: SQLite bindings
- `ai_models`: AI model bindings

## Trigger Types

### HTTP Trigger
```toml
[[trigger.http]]
component = "component-id"
route = "/path/*"
id = "unique-trigger-id"
```

### Redis Trigger
```toml
[[trigger.redis]]
component = "component-id"
channel = "channel-name"
id = "unique-trigger-id"
```

## Notes

- Each trigger must have a unique `id`
- Component IDs are used as keys in dotted notation
- Build commands are optional but recommended for source projects