//! CLI interface for tools command

use anyhow::Result;
use ftl_runtime::deps::{UserInterface, ProgressIndicator, MultiProgressManager, MessageStyle};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use crate::commands::tools::{ToolsDependencies, add_with_deps, list_with_deps, remove_with_deps, update_with_deps};

#[derive(Debug, Clone)]
pub enum ToolsCommand {
    /// List available pre-built tools
    List {
        /// Filter by category
        category: Option<String>,
        /// Filter by keyword
        filter: Option<String>,
        /// Registry to use
        registry: Option<String>,
        /// Show additional details
        verbose: bool,
        /// List from all enabled registries
        all: bool,
        /// Query registry directly
        direct: bool,
    },
    /// Add pre-built tools to your project
    Add {
        /// Tool names to add
        tools: Vec<String>,
        /// Registry to use
        registry: Option<String>,
        /// Version/tag to use
        version: Option<String>,
        /// Skip confirmation prompt
        yes: bool,
    },
    /// Update existing tools in your project
    Update {
        /// Tool names to update
        tools: Vec<String>,
        /// Registry to use
        registry: Option<String>,
        /// Version/tag to update to
        version: Option<String>,
        /// Skip confirmation prompt
        yes: bool,
    },
    /// Remove tools from your project
    Remove {
        /// Tool names to remove
        tools: Vec<String>,
        /// Skip confirmation prompt
        yes: bool,
    },
}

#[derive(Debug)]
pub struct ToolsArgs {
    pub command: ToolsCommand,
}

/// Execute tools command with proper dependency injection
pub async fn execute(args: ToolsArgs) -> Result<()> {
    // Create dependencies
    let ui: Arc<dyn UserInterface> = Arc::new(ConsoleUserInterface);
    let client = Client::new();
    
    let deps = Arc::new(ToolsDependencies {
        ui,
        client,
    });

    match args.command {
        ToolsCommand::List { category, filter, registry, verbose, all, direct } => {
            list_with_deps(
                &deps,
                category.as_deref(),
                filter.as_deref(),
                registry.as_deref(),
                verbose,
                all,
                direct,
            ).await
        }
        ToolsCommand::Add { tools, registry, version, yes } => {
            add_with_deps(
                &deps,
                &tools,
                registry.as_deref(),
                version.as_deref(),
                yes,
            ).await
        }
        ToolsCommand::Update { tools, registry, version, yes } => {
            update_with_deps(
                &deps,
                &tools,
                registry.as_deref(),
                version.as_deref(),
                yes,
            ).await
        }
        ToolsCommand::Remove { tools, yes } => {
            remove_with_deps(
                &deps,
                &tools,
                yes,
            ).await
        }
    }
}

/// Simple console user interface implementation
struct ConsoleUserInterface;

impl UserInterface for ConsoleUserInterface {
    fn create_spinner(&self) -> Box<dyn ProgressIndicator> {
        // Simple spinner implementation - for now just a placeholder
        Box::new(SimpleSpinner)
    }

    fn create_multi_progress(&self) -> Box<dyn MultiProgressManager> {
        // Simple multi-progress implementation - for now just a placeholder
        Box::new(SimpleMultiProgress)
    }

    fn print(&self, message: &str) {
        println!("{}", message);
    }

    fn print_styled(&self, message: &str, _style: MessageStyle) {
        println!("{}", message);
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn prompt_input(&self, prompt: &str, _default: Option<&str>) -> Result<String> {
        print!("{}", prompt);
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn prompt_select(&self, prompt: &str, items: &[&str], default: usize) -> Result<usize> {
        println!("{}", prompt);
        for (i, item) in items.iter().enumerate() {
            let marker = if i == default { "*" } else { " " };
            println!("{} {}: {}", marker, i, item);
        }
        
        print!("Selection [{}]: ", default);
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().is_empty() {
            Ok(default)
        } else {
            input.trim().parse().map_err(|e| anyhow::anyhow!("Invalid selection: {}", e))
        }
    }

    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }
}

/// Simple spinner implementation
struct SimpleSpinner;

impl ProgressIndicator for SimpleSpinner {
    fn set_message(&self, _message: &str) {}
    
    fn finish_and_clear(&self) {}
    
    fn enable_steady_tick(&self, _duration: Duration) {}
    
    fn finish_with_message(&self, _message: String) {}
    
    fn set_prefix(&self, _prefix: String) {}
}

/// Simple multi-progress implementation
struct SimpleMultiProgress;

impl MultiProgressManager for SimpleMultiProgress {
    fn add_spinner(&self) -> Box<dyn ProgressIndicator> {
        Box::new(SimpleSpinner)
    }
}