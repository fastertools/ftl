//! Engine management commands

use std::sync::Arc;

use anyhow::{Result, anyhow};
use uuid;

use ftl_runtime::api_client::types;
use ftl_runtime::deps::{FtlApiClient, MessageStyle, UserInterface};

/// Dependencies for the engine command
pub struct EngDependencies {
    /// User interface for output and interaction
    pub ui: Arc<dyn UserInterface>,
    /// API client for making requests to the FTL service
    pub api_client: Arc<dyn FtlApiClient>,
}

/// Output format for list command
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Display output as a formatted table
    Table,
    /// Display output as JSON
    Json,
}

/// Execute the list subcommand
pub async fn list_with_deps(format: OutputFormat, deps: &Arc<EngDependencies>) -> Result<()> {
    let response = deps.api_client.list_apps(None, None, None).await?;

    if response.apps.is_empty() {
        deps.ui
            .print_styled("No engines found.", MessageStyle::Yellow);
        return Ok(());
    }

    match format {
        OutputFormat::Table => display_apps_table(&response.apps, deps),
        OutputFormat::Json => display_apps_json(&response.apps, deps)?,
    }

    Ok(())
}

/// Execute the status subcommand
pub async fn status_with_deps(
    engine_id: &str,
    format: OutputFormat,
    deps: &Arc<EngDependencies>,
) -> Result<()> {
    // Try to parse as UUID first
    let engine_info = if uuid::Uuid::parse_str(engine_id).is_ok() {
        // It's a valid UUID, use get_app
        deps.api_client.get_app(engine_id).await?
    } else {
        // Not a UUID, assume it's a engine name and use list_apps with name filter
        let response = deps
            .api_client
            .list_apps(None, None, Some(engine_id))
            .await?;

        if response.apps.is_empty() {
            return Err(anyhow!("Engine '{}' not found", engine_id));
        }

        // Use list_apps with name filter to find the engine, then get full details
        let engine_uuid = &response.apps[0].app_id.to_string();
        deps.api_client.get_app(engine_uuid).await?
    };

    match format {
        OutputFormat::Table => display_engine_status_table(&engine_info, deps),
        OutputFormat::Json => display_engine_status_json(&engine_info, deps)?,
    }

    Ok(())
}

/// Execute the delete subcommand
pub async fn delete_with_deps(
    engine_id: &str,
    force: bool,
    deps: &Arc<EngDependencies>,
) -> Result<()> {
    // Get app status first to show what will be deleted
    // Try to parse as UUID first
    let engine_info = if uuid::Uuid::parse_str(engine_id).is_ok() {
        // It's a valid UUID, use get_app
        deps.api_client.get_app(engine_id).await?
    } else {
        // Not a UUID, assume it's a engine name and use list_apps with name filter
        let response = deps
            .api_client
            .list_apps(None, None, Some(engine_id))
            .await?;

        if response.apps.is_empty() {
            return Err(anyhow!("Engine '{}' not found", engine_id));
        }

        // Use list_apps with name filter to find the engine, then get full details
        let engine_uuid = &response.apps[0].app_id.to_string();
        deps.api_client.get_app(engine_uuid).await?
    };

    deps.ui
        .print_styled("Engine to be deleted:", MessageStyle::Yellow);
    let name = &engine_info.app_name;
    let id = &engine_info.app_id;
    deps.ui.print(&format!("  Name: {name}"));
    deps.ui.print(&format!("  ID: {id}"));
    if let Some(url) = &engine_info.provider_url {
        deps.ui.print(&format!("  URL: {url}"));
    }
    deps.ui.print("");

    if !force && deps.ui.is_interactive() {
        deps.ui
            .print_styled("⚠️  This action cannot be undone!", MessageStyle::Warning);

        let prompt = format!("Type '{name}' to confirm deletion");
        let input = deps.ui.prompt_input(&prompt, None)?;

        if input != *name {
            deps.ui
                .print_styled("Deletion cancelled.", MessageStyle::Yellow);
            return Ok(());
        }
    }

    deps.ui
        .print_styled("Deleting engine...", MessageStyle::Cyan);

    // Always use the UUID for deletion
    let response = deps
        .api_client
        .delete_app(&engine_info.app_id.to_string())
        .await?;

    let msg = &response.message;
    deps.ui
        .print_styled(&format!("✓ {msg}"), MessageStyle::Success);

    Ok(())
}

// Helper functions

fn display_apps_table(apps: &[types::ListAppsResponseAppsItem], deps: &Arc<EngDependencies>) {
    deps.ui.print("");

    for app in apps {
        // Print app name and ID
        deps.ui.print_styled(&app.app_name, MessageStyle::Bold);
        deps.ui.print(&format!("  ID: {}", &app.app_id));

        // Print status
        let status_text = app.status.to_string();
        deps.ui.print(&format!("  Status: {status_text}"));

        // Print URL if available
        if let Some(url) = &app.provider_url {
            deps.ui.print(&format!("  URL: {url}"));
        }

        deps.ui.print("");
    }

    let count = apps.len();
    let plural = if count == 1 { "" } else { "s" };
    deps.ui.print(&format!("Total: {count} engine{plural}"));
}

fn display_apps_json(
    apps: &[types::ListAppsResponseAppsItem],
    deps: &Arc<EngDependencies>,
) -> Result<()> {
    let json = serde_json::to_string_pretty(&apps)?;
    deps.ui.print(&json);
    Ok(())
}

fn display_engine_status_table(engine_info: &types::App, deps: &Arc<EngDependencies>) {
    deps.ui.print("");
    deps.ui.print_styled("Engine Details", MessageStyle::Bold);
    let name = &engine_info.app_name;
    deps.ui.print(&format!("  Name:         {name}"));
    let id = &engine_info.app_id;
    deps.ui.print(&format!("  ID:           {id}"));

    let status_text = engine_info.status.to_string();
    deps.ui.print(&format!("  Status:       {status_text}"));

    if let Some(url) = &engine_info.provider_url {
        deps.ui.print(&format!("  URL:          {url}"));
    }

    if let Some(error) = &engine_info.provider_error {
        deps.ui
            .print_styled(&format!("  Error:        {error}"), MessageStyle::Red);
    }

    deps.ui
        .print(&format!("  Created:      {}", engine_info.created_at));
    deps.ui
        .print(&format!("  Updated:      {}", engine_info.updated_at));

    deps.ui.print("");
}

fn display_engine_status_json(engine_info: &types::App, deps: &Arc<EngDependencies>) -> Result<()> {
    let json = serde_json::to_string_pretty(engine_info)?;
    deps.ui.print(&json);
    Ok(())
}

/// Engine command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct EngineArgs {
    /// Subcommand
    pub command: EngineCommand,
}

/// Application subcommands
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// List all applications
    List {
        /// Output format
        format: OutputFormat,
    },
    /// Get status of an application
    Status {
        /// Application ID
        app_id: String,
        /// Output format
        format: OutputFormat,
    },
    /// Delete an application
    Delete {
        /// Application ID
        app_id: String,
        /// Force deletion without confirmation
        force: bool,
    },
}

impl OutputFormat {
    /// Parse output format from string
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            _ => anyhow::bail!("Invalid output format: {}", s),
        }
    }
}

/// Execute the engine command with default dependencies
pub async fn execute(args: EngineArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_runtime::deps::{CredentialsProvider, RealCredentialsProvider, RealFtlApiClient};

    // Get credentials first to create authenticated API client
    let credentials_provider = RealCredentialsProvider;
    let credentials = credentials_provider.get_or_refresh_credentials().await?;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(EngDependencies {
        ui: ui.clone(),
        api_client: Arc::new(RealFtlApiClient::new_with_auth(
            ftl_runtime::api_client::Client::new(ftl_runtime::config::DEFAULT_API_BASE_URL),
            credentials.access_token,
        )),
    });

    match args.command {
        EngineCommand::List { format } => list_with_deps(format, &deps).await,
        EngineCommand::Status { app_id, format } => status_with_deps(&app_id, format, &deps).await,
        EngineCommand::Delete { app_id, force } => delete_with_deps(&app_id, force, &deps).await,
    }
}

#[cfg(test)]
#[path = "eng_tests.rs"]
mod tests;
