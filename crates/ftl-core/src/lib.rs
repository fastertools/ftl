//! Core functionality for the FTL CLI
//!
//! This crate contains the foundational types and functionality used across
//! the FTL CLI application, including configuration management, API client
//! implementations, and dependency injection interfaces.

/// API client module for interacting with the FTL backend
pub mod api_client;
/// Configuration constants and types
pub mod config;
/// Dependency injection traits and implementations
pub mod deps;

#[cfg(test)]
pub mod test_helpers;

// Re-export commonly used types at the crate root
pub use config::{DEFAULT_API_BASE_URL, API_URL_ENV_VAR, AUTH_TOKEN_ENV_VAR, DEFAULT_API_TIMEOUT_SECS};
pub use deps::{
    StoredCredentials, FileSystem, CommandExecutor, CommandOutput, FtlApiClient,
    Clock, CredentialsProvider, UserInterface, ProgressIndicator, MultiProgressManager,
    MessageStyle, AsyncRuntime, ProcessManager, ProcessHandle,
    ExitStatus, RealFileSystem, RealCommandExecutor, RealFtlApiClient, RealClock,
    RealCredentialsProvider, RealAsyncRuntime, RealProcessManager
};