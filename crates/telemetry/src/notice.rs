//! First-run notice system for telemetry
//!
//! This module handles displaying a privacy-friendly notice to users
//! on their first run of the CLI, informing them about telemetry
//! and how to opt out.

use anyhow::Result;
use ftl_common::config::{Config, ConfigSection};
use serde::{Deserialize, Serialize};
use std::io;

/// First-run notice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeConfig {
    /// Whether the notice has been shown
    #[serde(default)]
    pub notice_shown: bool,

    /// Version of the notice that was shown
    #[serde(default = "default_notice_version")]
    pub notice_version: String,
}

fn default_notice_version() -> String {
    "1.0".to_string()
}

impl Default for NoticeConfig {
    fn default() -> Self {
        Self {
            notice_shown: false,
            notice_version: "1.0".to_string(),
        }
    }
}

impl ConfigSection for NoticeConfig {
    fn section_name() -> &'static str {
        "telemetry_notice"
    }
}

/// Current version of the notice
const NOTICE_VERSION: &str = "1.0";

/// The telemetry notice text
const TELEMETRY_NOTICE: &str = r"
╭─────────────────────────────────────────────────────────────────╮
│                    FTL CLI Telemetry Notice                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  FTL CLI collects anonymous usage data to help improve the      │
│  tool. This data is stored locally and shared with FTL to       │
│  help us understand usage patterns and improve the platform.    │
│                                                                 │
│  We collect:                                                    │
│  • Command usage (which commands are run)                       │
│  • Command arguments (with sensitive values filtered)           │
│  • Command success/failure rates                                │
│  • Performance metrics (execution time)                         │
│  • Operating system and architecture                            │
│                                                                 │
│  We do NOT collect:                                             │
│  • Personal information                                         │
│  • Passwords, tokens, or API keys                               │
│  • File contents or project data                                │
│                                                                 │
│  To opt out of telemetry:                                       │
│  • Run: ftl telemetry disable                                   │
│  • Or set: FTL_TELEMETRY_DISABLED=1                             │
│                                                                 │
│  Learn more: https://github.com/fastertools/ftl-cli             │
│                                                                 │
╰─────────────────────────────────────────────────────────────────╯
";

/// Check if we should show the first-run notice
pub fn should_show_notice() -> Result<bool> {
    // If telemetry is disabled via env var, don't show notice
    if std::env::var("FTL_TELEMETRY_DISABLED").is_ok() {
        return Ok(false);
    }

    // Check if we're in a CI environment
    if is_ci_environment() {
        return Ok(false);
    }

    // Check if notice has been shown
    let config = Config::load()?;

    // Try to get the notice config - if it fails to deserialize, treat as first run
    let Ok(Some(notice_config)) = config.get_section::<NoticeConfig>() else {
        return Ok(true); // No config or malformed config means show notice
    };

    // Show notice if not shown before or version has changed
    Ok(!notice_config.notice_shown || notice_config.notice_version != NOTICE_VERSION)
}

/// Display the first-run notice
pub fn show_notice() -> Result<()> {
    // Print the notice
    println!("{TELEMETRY_NOTICE}");

    // Add a small pause to ensure the user sees it
    println!("Press Enter to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Mark notice as shown
    mark_notice_shown()?;

    Ok(())
}

/// Display the notice without waiting for input (non-interactive mode)
pub fn show_notice_non_interactive() -> Result<()> {
    println!("{TELEMETRY_NOTICE}");
    mark_notice_shown()?;
    Ok(())
}

/// Mark the notice as shown in the config
fn mark_notice_shown() -> Result<()> {
    let mut config = Config::load()?;
    let notice_config = NoticeConfig {
        notice_shown: true,
        notice_version: NOTICE_VERSION.to_string(),
    };
    config.set_section(notice_config)?;
    config.save()?;
    Ok(())
}

/// Check if we're running in a CI environment
fn is_ci_environment() -> bool {
    // Common CI environment variables
    let ci_vars = [
        "CI",
        "CONTINUOUS_INTEGRATION",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "CIRCLECI",
        "TRAVIS",
        "JENKINS_URL",
        "TEAMCITY_VERSION",
        "TF_BUILD", // Azure DevOps
    ];

    ci_vars.iter().any(|var| std::env::var(var).is_ok())
}

/// Check if stdout is a terminal (for interactive detection)
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal() && std::io::stdin().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_notice_config_default() {
        let config = NoticeConfig::default();
        assert!(!config.notice_shown);
        assert_eq!(config.notice_version, "1.0");
    }

    #[test]
    fn test_should_show_notice_first_run() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create empty config
        let config = Config::load_from_path(&config_path).unwrap();
        config.save().unwrap();

        // First run should show notice (in test env, would check env vars)
        // We can't fully test this without mocking env vars
    }

    #[test]
    fn test_ci_environment_detection() {
        // This will be false in normal test environment
        assert!(!is_ci_environment());

        // Would need to mock env vars to test positive case
    }
}
