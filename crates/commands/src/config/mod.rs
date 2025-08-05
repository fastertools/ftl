//! Configuration types and utilities for FTL CLI

/// Registry configuration types
pub mod registry;

/// FTL-specific configuration management
pub mod ftl_config;

/// Transpiler for converting ftl.toml to spin.toml
pub mod transpiler;

/// Spin manifest configuration types
pub mod spin_config;

pub use ftl_config::{FtlConfig, ToolConfig};
pub use registry::{RegistryConfig, RegistryType};
pub use transpiler::transpile_ftl_to_spin;
