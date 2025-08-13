//! FTL Resolve Library
//!
//! This library provides component resolution and transpilation for FTL configuration files:
//! downloads registry components using wkg, validates syntax, and transpiles to Spin TOML format.

pub mod ftl_resolve;
pub mod resolver;
pub mod spin_config;
pub mod transpiler;

pub use ftl_resolve::{
    ApplicationVariable, BuildConfig, ComponentConfig, FtlConfig, McpConfig, OauthConfig,
    ProjectConfig,
};
pub use resolver::{
    check_wkg_available, resolve_all_components, resolve_and_transpile, resolve_registry_component,
};
pub use transpiler::{
    create_spin_toml_with_resolved_paths, transpile_ftl_to_spin, validate_local_auth,
};

// Re-export for convenience
pub use schemars::schema_for;
