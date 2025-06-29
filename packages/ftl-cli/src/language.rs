use std::{fmt, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{manifest::Manifest, templates::Template};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    JavaScript,
}

impl Default for Language {
    fn default() -> Self {
        Language::Rust
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::JavaScript => write!(f, "javascript"),
        }
    }
}

impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" => Some(Language::Rust),
            "javascript" | "js" => Some(Language::JavaScript),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn detect_from_path(path: &Path) -> Option<Self> {
        // Check for language-specific files
        if path.join("Cargo.toml").exists() {
            return Some(Language::Rust);
        }
        if path.join("package.json").exists() {
            return Some(Language::JavaScript);
        }
        None
    }

    #[allow(dead_code)]
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::JavaScript => "js",
        }
    }

    #[allow(dead_code)]
    pub fn source_dir(&self) -> &'static str {
        "src"
    }

    #[allow(dead_code)]
    pub fn build_output_path(&self) -> &'static str {
        match self {
            Language::Rust => "target/wasm32-wasip1/release",
            Language::JavaScript => "target",
        }
    }

    #[allow(dead_code)]
    pub fn wasm_file_name(&self) -> &'static str {
        match self {
            Language::Rust => "{name}.wasm",
            Language::JavaScript => "tool.wasm",
        }
    }
}

pub trait LanguageSupport: Send + Sync {
    #[allow(dead_code)]
    fn language(&self) -> Language;
    fn new_project(&self, name: &str, description: &str, template: &str, path: &Path)
    -> Result<()>;
    fn build(&self, manifest: &Manifest, path: &Path) -> Result<()>;
    fn test(&self, manifest: &Manifest, path: &Path) -> Result<()>;
    #[allow(dead_code)]
    fn get_templates(&self) -> Vec<Template>;
    fn validate_environment(&self) -> Result<()>;
}

pub mod javascript;
pub mod rust;

use self::{javascript::JavaScriptSupport, rust::RustSupport};

pub fn get_language_support(language: Language) -> Box<dyn LanguageSupport> {
    match language {
        Language::Rust => Box::new(RustSupport::new()),
        Language::JavaScript => Box::new(JavaScriptSupport::new()),
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

    #[allow(dead_code)]
    pub fn install_command(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm install",
            PackageManager::Yarn => "yarn install",
            PackageManager::Pnpm => "pnpm install",
        }
    }

    pub fn run_command(&self, script: &str) -> String {
        match self {
            PackageManager::Npm => format!("npm run {}", script),
            PackageManager::Yarn => format!("yarn {}", script),
            PackageManager::Pnpm => format!("pnpm {}", script),
        }
    }

    #[allow(dead_code)]
    pub fn exec_command(&self, cmd: &str) -> String {
        match self {
            PackageManager::Npm => format!("npx {}", cmd),
            PackageManager::Yarn => format!("yarn {}", cmd),
            PackageManager::Pnpm => format!("pnpm exec {}", cmd),
        }
    }
}
