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
pub use config::{
    API_URL_ENV_VAR, AUTH_TOKEN_ENV_VAR, DEFAULT_API_BASE_URL, DEFAULT_API_TIMEOUT_SECS,
};
pub use deps::{
    AsyncRuntime, Clock, CommandExecutor, CommandOutput, CredentialsProvider, ExitStatus,
    FileSystem, FtlApiClient, MessageStyle, MultiProgressManager, ProcessHandle, ProcessManager,
    ProgressIndicator, RealAsyncRuntime, RealClock, RealCommandExecutor, RealCredentialsProvider,
    RealFileSystem, RealFtlApiClient, RealProcessManager, StoredCredentials, UserInterface,
};
