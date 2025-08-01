use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported programming languages for toolboxes
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
    /// Python programming language
    Python,
    /// Go programming language
    Go,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "javascript" | "js" => Ok(Self::JavaScript),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "python" | "py" => Ok(Self::Python),
            "go" | "golang" => Ok(Self::Go),
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
            Self::Python => write!(f, "Python"),
            Self::Go => write!(f, "Go"),
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
        assert_eq!(Language::from_str("python"), Ok(Language::Python));
        assert_eq!(Language::from_str("py"), Ok(Language::Python));
        assert_eq!(Language::from_str("go"), Ok(Language::Go));
        assert_eq!(Language::from_str("golang"), Ok(Language::Go));
        assert!(Language::from_str("unknown").is_err());
    }
}
