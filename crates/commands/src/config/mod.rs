//! Configuration types and utilities for FTL CLI

/// Registry configuration types
pub mod registry;

/// FTL-specific configuration management
pub mod ftl_config;

/// Transpiler for converting ftl.toml to spin.toml
pub mod transpiler;

pub use registry::{RegistryConfig, RegistryType};
pub use ftl_config::{FtlConfig, ToolConfig, AuthConfig};
pub use transpiler::transpile_ftl_to_spin;
