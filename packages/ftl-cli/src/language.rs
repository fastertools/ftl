use std::{fmt, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::manifest::Manifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Rust,
    JavaScript,
    TypeScript,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::JavaScript => write!(f, "javascript"),
            Language::TypeScript => write!(f, "typescript"),
        }
    }
}

impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" => Some(Language::Rust),
            "javascript" | "js" => Some(Language::JavaScript),
            "typescript" | "ts" => Some(Language::TypeScript),
            _ => None,
        }
    }
}

pub trait LanguageSupport: Send + Sync {
    fn new_project(&self, name: &str, description: &str, template: &str, path: &Path)
    -> Result<()>;
    fn build(&self, manifest: &Manifest, path: &Path) -> Result<()>;
    fn test(&self, manifest: &Manifest, path: &Path) -> Result<()>;
    fn validate_environment(&self) -> Result<()>;
}

pub mod javascript;
pub mod rust;
pub mod typescript;

use self::{javascript::JavaScriptSupport, rust::RustSupport, typescript::TypeScriptSupport};

pub fn get_language_support(language: Language) -> Box<dyn LanguageSupport> {
    match language {
        Language::Rust => Box::new(RustSupport::new()),
        Language::JavaScript => Box::new(JavaScriptSupport::new()),
        Language::TypeScript => Box::new(TypeScriptSupport::new()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
}

impl PackageManager {
    pub fn detect(path: &Path) -> Self {
        if path.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if path.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else {
            PackageManager::Npm
        }
    }

    pub fn run_command(&self, script: &str) -> String {
        match self {
            PackageManager::Npm => format!("npm run {script}"),
            PackageManager::Yarn => format!("yarn {script}"),
            PackageManager::Pnpm => format!("pnpm {script}"),
        }
    }
}
