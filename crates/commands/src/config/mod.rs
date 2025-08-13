//! Configuration types and utilities for FTL CLI

/// Registry configuration types
pub mod registry;

/// Path resolution utilities for transpilation
pub mod path_resolver;

/// User configuration management
pub mod user_config;

pub use ftl_resolve::{ComponentConfig, FtlConfig};
pub use path_resolver::create_spin_toml_with_resolved_paths;
pub use registry::{RegistryConfig, RegistryType};
pub use user_config::UserConfig;

// Re-export from ftl-resolve crate
pub use ftl_resolve::transpile_ftl_to_spin;
