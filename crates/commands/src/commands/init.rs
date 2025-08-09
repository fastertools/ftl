//! Refactored init command with dependency injection for better testability

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};

use crate::config::ftl_config::{FtlConfig, McpConfig, ProjectConfig};
use ftl_common::{RealUserInterface, SpinInstaller, check_and_install_spin};
use ftl_runtime::deps::{
    CommandExecutor, FileSystem, RealCommandExecutor, RealFileSystem, UserInterface,
};

/// Init command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct InitArgs {
    /// Name of the new project
    pub name: Option<String>,
    /// Initialize in current directory instead of creating new one
    pub here: bool,
}

/// Init command configuration
pub struct InitConfig {
    /// Name of the new project
    pub name: Option<String>,
    /// Initialize in current directory instead of creating new one
    pub here: bool,
}

/// Dependencies for the init command
pub struct InitDependencies {
    /// File system operations
    pub file_system: Arc<dyn FileSystem>,
    /// Command execution operations
    pub command_executor: Arc<dyn CommandExecutor>,
    /// User interface for output and prompts
    pub ui: Arc<dyn UserInterface>,
    /// Spin CLI installer
    pub spin_installer: Arc<dyn SpinInstaller>,
}

/// Execute the init command with injected dependencies
pub async fn execute_with_deps(config: InitConfig, deps: Arc<InitDependencies>) -> Result<()> {
    let InitConfig { mut name, here } = config;

    // Install Spin if needed
    let _spin_path = deps.spin_installer.check_and_install().await?;

    // Get project name
    if name.is_none() && !here {
        name = Some(deps.ui.prompt_input("Project name", Some("my-project"))?);
    }

    // Validate name
    if let Some(ref project_name) = name {
        validate_project_name(project_name)?;
    }

    // Check directory
    let target_dir = if here {
        ".".to_string()
    } else {
        name.as_ref().unwrap().clone()
    };

    if !here && deps.file_system.exists(Path::new(&target_dir)) {
        anyhow::bail!("Directory '{}' already exists", target_dir);
    }

    if here && !is_directory_empty(&deps.file_system) {
        anyhow::bail!("Current directory is not empty. Use --here only in an empty directory.");
    }

    // Create project with ftl.toml
    create_ftl_project(&deps.file_system, &target_dir, name.as_deref())?;

    // Success message
    deps.ui.print("");
    deps.ui.print("✅ MCP project initialized!");
    deps.ui.print("");
    deps.ui.print("Next steps:");

    if !here {
        deps.ui.print(&format!("  cd {target_dir}"));
    }

    deps.ui
        .print("  ftl add           # Add a tool to the project");
    deps.ui.print("  ftl build         # Build the project");
    deps.ui
        .print("  ftl up            # Start local dev server");
    deps.ui.print("");
    deps.ui.print("The project will be available at:");
    deps.ui.print("  http://localhost:3000/mcp");
    deps.ui.print("");

    Ok(())
}

/// Validate project name
fn validate_project_name(name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "Project name cannot be empty");

    ensure!(
        name.chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '-'),
        "Project name must be lowercase alphanumeric with hyphens"
    );

    ensure!(
        !name.starts_with('-') && !name.ends_with('-'),
        "Project name cannot start or end with hyphens"
    );

    ensure!(
        !name.contains("--"),
        "Project name cannot contain consecutive hyphens"
    );

    Ok(())
}

/// Check if current directory is empty
fn is_directory_empty(fs: &Arc<dyn FileSystem>) -> bool {
    let common_files = [
        "./Cargo.toml",
        "./package.json",
        "./spin.toml",
        "./.git",
        "./src",
        "./components",
        "./node_modules",
    ];

    for file in &common_files {
        if fs.exists(Path::new(file)) {
            return false;
        }
    }

    true
}

/// Create FTL project with ftl.toml
fn create_ftl_project(
    fs: &Arc<dyn FileSystem>,
    target_dir: &str,
    project_name: Option<&str>,
) -> Result<()> {
    use std::collections::HashMap;

    // Create project directory if not using --here
    if target_dir != "." {
        std::fs::create_dir_all(target_dir).context("Failed to create project directory")?;
    }

    let project_path = Path::new(target_dir);
    let name = project_name.unwrap_or(target_dir);

    // Create basic ftl.toml
    let config = FtlConfig {
        project: ProjectConfig {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: "FTL MCP server for hosting MCP tools".to_string(),
            authors: vec![],
            access_control: "public".to_string(),
            default_registry: None,
        },
        oauth: None,
        component: HashMap::new(),
        mcp: McpConfig::default(),
        variables: HashMap::new(),
    };

    let ftl_content = config.to_toml_string()?;
    fs.write_string(&project_path.join("ftl.toml"), &ftl_content)
        .context("Failed to write ftl.toml")?;

    // Create README.md
    let readme_content = format!(
        r"# {name}

FTL MCP server for hosting MCP tools.

## Getting Started

### Scaffold a new tool:
```bash
ftl add my-tool --language rust
```

## Running the server

### Basic

```bash
ftl build && ftl up
```

### Hot Reloading with `watch`

```bash
ftl up --watch

→ Starting development server with auto-rebuild...

👀 Watching for file changes

Serving http://127.0.0.1:3000
Available Routes:
  mcp: http://127.0.0.1:3000 (wildcard)
```

## Deployment Options

### Export spin.toml

```bash
ftl build --export-out ./spin.toml --export spin
```

This allows you to run the application with `spin` or deploy to any platform that supports it.

### FTL Engine

#### Login to FTL Engine

```bash
ftl eng login
```

Deploy your MCP server:
```bash
ftl eng deploy
```

For more information, visit the [FTL documentation](https://docs.fastertools.com).
"
    );

    fs.write_string(&project_path.join("README.md"), &readme_content)
        .context("Failed to write README.md")?;

    // Create .gitignore
    let gitignore_content = r"# Dependencies
node_modules/
target/
dist/
.spin/
.ftl/

# Environment
.env
.env.local

# Build outputs
*.wasm
*.wat

# Generated files
# spin.toml is auto-generated from ftl.toml
spin.toml

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
";

    fs.write_string(&project_path.join(".gitignore"), gitignore_content)
        .context("Failed to write .gitignore")?;

    Ok(())
}

// Spin installer wrapper that adapts the common implementation
struct SpinInstallerWrapper;

#[async_trait::async_trait]
impl SpinInstaller for SpinInstallerWrapper {
    async fn check_and_install(&self) -> Result<String> {
        let path = check_and_install_spin().await?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// Execute the init command with default dependencies
pub async fn execute(args: InitArgs) -> Result<()> {
    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(InitDependencies {
        file_system: Arc::new(RealFileSystem),
        command_executor: Arc::new(RealCommandExecutor),
        ui: ui.clone(),
        spin_installer: Arc::new(SpinInstallerWrapper),
    });

    let config = InitConfig {
        name: args.name,
        here: args.here,
    };

    execute_with_deps(config, deps).await
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
