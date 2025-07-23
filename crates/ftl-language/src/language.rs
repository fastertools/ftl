use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported programming languages for FTL projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Language {
    /// Rust programming language
    Rust,
    /// JavaScript programming language
    JavaScript,
    /// TypeScript programming language
    #[default]
    TypeScript,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "javascript" | "js" => Ok(Self::JavaScript),
            "typescript" | "ts" => Ok(Self::TypeScript),
            _ => Err(format!("Unknown language: {s}")),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::JavaScript => write!(f, "JavaScript"),
            Self::TypeScript => write!(f, "TypeScript"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("rust"), Ok(Language::Rust));
        assert_eq!(Language::from_str("rs"), Ok(Language::Rust));
        assert_eq!(Language::from_str("javascript"), Ok(Language::JavaScript));
        assert_eq!(Language::from_str("js"), Ok(Language::JavaScript));
        assert_eq!(Language::from_str("typescript"), Ok(Language::TypeScript));
        assert_eq!(Language::from_str("ts"), Ok(Language::TypeScript));
        assert!(Language::from_str("unknown").is_err());
    }
}
