# FTL Resolveuration System

The FTL Resolveuration system provides a flexible, type-safe way to manage application configuration across different components of the FTL CLI.

## Overview

The configuration system stores data in `~/.ftl/config.toml` as a TOML file where each top-level table represents a different configuration section. Each section is managed independently by the component that owns it.

## Quick Start

### 1. Define Your Configuration Section

```rust
use serde::{Deserialize, Serialize};
use ftl_common::config::{Config, ConfigSection};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyConfig {
    enabled: bool,
    api_key: Option<String>,
}

impl ConfigSection for MyConfig {
    fn section_name() -> &'static str {
        "my_feature"
    }
}
```

### 2. Load and Use Configuration

```rust
use ftl_common::config::Config;

// Load configuration
let mut config = Config::load()?;

// Get your section
let my_config = config.get_section::<MyConfig>()?
    .unwrap_or_else(|| MyConfig {
        enabled: true,
        api_key: None,
    });

// Update and save
let updated = MyConfig {
    enabled: false,
    api_key: Some("new-key".to_string()),
};
config.set_section(updated)?;
config.save()?;
```

## Key Features

- **Type Safety**: Each configuration section is a strongly-typed Rust struct
- **Isolation**: Different components can manage their own sections without conflicts
- **Atomicity**: All changes are saved atomically to prevent corruption
- **Extensibility**: New sections can be added without modifying existing code
- **Simplicity**: Clean API with sensible defaults

## API Reference

### Core Types

#### `ConfigSection` Trait
Every configuration section must implement this trait:

```rust
pub trait ConfigSection: Serialize + Deserialize + Clone {
    fn section_name() -> &'static str;
}
```

#### `Config` Struct
The main configuration manager:

```rust
impl Config {
    // Load from default location (~/.ftl/config.toml)
    pub fn load() -> Result<Self>
    
    // Load from custom path
    pub fn load_from_path(path: &Path) -> Result<Self>
    
    // Get a configuration section
    pub fn get_section<T: ConfigSection>(&self) -> Result<Option<T>>
    
    // Set a configuration section
    pub fn set_section<T: ConfigSection>(&mut self, section: T) -> Result<()>
    
    // Remove a configuration section
    pub fn remove_section<T: ConfigSection>(&mut self) -> Option<Value>
    
    // Check if a section exists
    pub fn has_section<T: ConfigSection>(&self) -> bool
    
    // Save to disk
    pub fn save(&self) -> Result<()>
}
```

### Helper Functions

```rust
// Load or create a section with default values
pub fn load_or_create_section<T: ConfigSection + Default>() -> Result<(T, bool)>
```

## Common Patterns

### Pattern 1: Configuration with Defaults

```rust
impl Default for MyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_secs: 30,
            max_retries: 3,
        }
    }
}

// Usage
let (config, was_created) = load_or_create_section::<MyConfig>()?;
if was_created {
    println!("Created config with defaults");
}
```

### Pattern 2: Optional Configuration

```rust
let config = Config::load()?;
if let Some(my_config) = config.get_section::<MyConfig>()? {
    // Use custom configuration
} else {
    // Use hardcoded defaults
}
```

### Pattern 3: Feature Flags

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeatureFlags {
    experimental_feature: bool,
    beta_api: bool,
}

impl ConfigSection for FeatureFlags {
    fn section_name() -> &'static str {
        "features"
    }
}
```

### Pattern 4: Managing Credentials

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Credentials {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
}

// Clear credentials on logout
fn logout() -> Result<()> {
    let mut config = Config::load()?;
    config.remove_section::<Credentials>();
    config.save()?;
    Ok(())
}
```

## File Structure

The configuration file is stored at `~/.ftl/config.toml` with the following structure:

```toml
[telemetry]
installation_id = "550e8400-e29b-41d4-a716-446655440000"
telemetry_enabled = true

[auth]
access_token = "..."
refresh_token = "..."
expires_at = 1234567890

[registry]
default_registry = "ghcr.io"

[[registry.registries]]
name = "ghcr.io"
url = "https://ghcr.io"
auth_required = true
```

## Error Handling

The configuration system uses `anyhow::Result` for error handling. Common errors include:

- **File not found**: Automatically handled by creating a new config
- **Parse errors**: Invalid JSON in the config file
- **Permission errors**: Cannot read/write the config file
- **Serialization errors**: Invalid data types

## Best Practices

1. **Define a `Default` implementation** for your config sections when possible
2. **Use descriptive section names** that clearly identify the feature
3. **Document your configuration fields** for other developers
4. **Handle missing sections gracefully** with sensible defaults
5. **Validate configuration values** after loading
6. **Don't store sensitive data in plain text** - consider using the system keyring

## Examples

See `example.rs` in this directory for comprehensive examples covering:
- Basic usage
- Multiple sections
- Updating specific fields
- Checking section existence
- Removing sections
- Custom file paths

## Testing

When testing code that uses the configuration system:

```rust
use tempfile::TempDir;

#[test]
fn test_my_feature() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    let mut config = Config::load_from_path(&config_path).unwrap();
    // Test your configuration logic
}
```