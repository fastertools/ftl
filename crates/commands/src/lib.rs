//! Command implementations for the FTL CLI
//!
//! This crate contains all the CLI command implementations for FTL,
//! including project initialization, building, deployment, and more.

/// Command implementations module
pub mod commands;

#[cfg(test)]
pub mod test_helpers;

// Re-export all commands at the crate root for easier access
pub use commands::{
    add, app, auth, build, deploy, init, login, logout, publish, registry, setup, test, up, update,
};
