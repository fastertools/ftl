//! Language detection and project analysis for the FTL CLI
//!
//! This crate provides functionality for detecting the programming language
//! of a project and analyzing project structure.

/// Language detection and enumeration module
pub mod language;

// Re-export the main types
pub use language::Language;
