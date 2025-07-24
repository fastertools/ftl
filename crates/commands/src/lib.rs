//! Command implementations for the FTL CLI
//!
//! This crate contains all the CLI command implementations for FTL,
//! including project initialization, building, deployment, and more.

/// Command implementations module
pub mod commands;

/// Configuration types and utilities
pub mod config;

/// Registry infrastructure and adapters
pub mod registry;

#[cfg(test)]
pub mod test_helpers;

// Re-export all commands at the crate root for easier access
pub use commands::{
    add, app, auth, build, deploy, init, login, logout, publish, setup, test, up, update,
};

// Re-export registry command separately to avoid naming conflict with registry module
pub use commands::registry as registry_command;
