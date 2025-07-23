//! Common utilities for the FTL CLI
//!
//! This crate contains shared utilities and helper functions used across
//! the FTL CLI application, including UI helpers, Spin installation utilities,
//! and version caching.

pub mod spin_installer;
pub mod ui;
pub mod version_cache;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod spin_installer_tests;
#[cfg(test)]
mod spin_installer_tests_akamai;
#[cfg(test)]
mod version_cache_tests;

// Re-export commonly used utilities at the crate root
pub use spin_installer::{check_and_install_spin, SpinInstaller};
pub use ui::RealUserInterface;
pub use version_cache::{check_and_prompt_for_update, VersionCache};