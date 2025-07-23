//! Application management commands

use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

use ftl_core::api_client::types;
use ftl_core::deps::{FtlApiClient, MessageStyle, UserInterface};

/// Dependencies for the app command
pub struct AppDependencies {
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

/// Application info for JSON output
#[derive(Serialize)]
struct AppInfo {
    name: String,
    url: String,
}

/// Execute the list subcommand
pub async fn list_with_deps(format: OutputFormat, deps: &Arc<AppDependencies>) -> Result<()> {
    let response = deps.api_client.list_apps().await?;

    if response.apps.is_empty() {
        deps.ui
            .print_styled("No applications found.", MessageStyle::Yellow);
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
    app_name: &str,
    format: OutputFormat,
    deps: &Arc<AppDependencies>,
) -> Result<()> {
    let response = deps.api_client.get_app_status(app_name).await?;

    match format {
        OutputFormat::Table => display_app_status_table(&response.app, deps),
        OutputFormat::Json => display_app_status_json(&response.app, deps)?,
    }

    Ok(())
}

/// Execute the delete subcommand
pub async fn delete_with_deps(
    app_name: &str,
    force: bool,
    deps: &Arc<AppDependencies>,
) -> Result<()> {
    // Get app status first to show what will be deleted
    let status = deps.api_client.get_app_status(app_name).await?;

    deps.ui
        .print_styled("Application to be deleted:", MessageStyle::Yellow);
    let name = &status.app.name;
    let url = &status.app.url;
    deps.ui.print(&format!("  Name: {name}"));
    deps.ui.print(&format!("  URL: {url}"));
    deps.ui.print("");

    if !force && deps.ui.is_interactive() {
        deps.ui
            .print_styled("⚠️  This action cannot be undone!", MessageStyle::Warning);

        let prompt = format!("Type '{app_name}' to confirm deletion");
        let input = deps.ui.prompt_input(&prompt, None)?;

        if input != app_name {
            deps.ui
                .print_styled("Deletion cancelled.", MessageStyle::Yellow);
            return Ok(());
        }
    }

    deps.ui
        .print_styled("Deleting application...", MessageStyle::Cyan);

    let response = deps.api_client.delete_app(app_name).await?;

    let msg = &response.message;
    deps.ui
        .print_styled(&format!("✓ {msg}"), MessageStyle::Success);

    if !response.warning.is_empty() {
        deps.ui.print("");
        deps.ui
            .print_styled(&response.warning, MessageStyle::Warning);
    }

    Ok(())
}

// Helper functions

fn display_apps_table(apps: &[types::ListAppsResponseAppsItem], deps: &Arc<AppDependencies>) {
    deps.ui.print("");

    for app in apps {
        // Print app name
        deps.ui.print_styled(&app.name, MessageStyle::Bold);

        // Print URL
        deps.ui.print(&format!("  {}", &app.url));
        deps.ui.print("");
    }

    let count = apps.len();
    let plural = if count == 1 { "" } else { "s" };
    deps.ui
        .print(&format!("Total: {count} application{plural}"));
}

fn display_apps_json(
    apps: &[types::ListAppsResponseAppsItem],
    deps: &Arc<AppDependencies>,
) -> Result<()> {
    let app_infos: Vec<AppInfo> = apps
        .iter()
        .map(|app| AppInfo {
            name: app.name.clone(),
            url: app.url.clone(),
        })
        .collect();

    let json = serde_json::to_string_pretty(&app_infos)?;
    deps.ui.print(&json);

    Ok(())
}

fn display_app_status_table(app: &types::GetAppStatusResponseApp, deps: &Arc<AppDependencies>) {
    deps.ui.print("");
    deps.ui
        .print_styled("Application Details", MessageStyle::Bold);
    let name = &app.name;
    deps.ui.print(&format!("  Name:         {name}"));
    let id = &app.id;
    deps.ui.print(&format!("  ID:           {id}"));
    let url = &app.url;
    deps.ui.print(&format!("  URL:          {url}"));

    let status_text = app
        .status
        .as_ref()
        .map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);
    deps.ui.print(&format!("  Status:       {status_text}"));

    if let Some(created) = &app.created_at {
        let created_str = format_datetime(created);
        deps.ui.print(&format!("  Created:      {created_str}"));
    }

    if let Some(last_deploy) = &app.last_deployment {
        let deploy_str = format_datetime(last_deploy);
        deps.ui.print(&format!("  Last Deploy:  {deploy_str}"));
    }

    let count = app.deployment_count;
    deps.ui.print(&format!("  Deployments:  {count}"));

    if let Some(invocations) = &app.invocations {
        deps.ui.print(&format!("  Invocations:  {invocations}"));
    }

    // Show last deployment details
    if let Some(last_info) = &app.last_deployment_info {
        deps.ui.print("");
        deps.ui.print_styled("Last Deployment", MessageStyle::Bold);
        let deployment_id = &last_info.deployment_id;
        deps.ui.print(&format!("  ID:           {deployment_id}"));
        let status_str = last_info.status.to_string();
        deps.ui.print(&format!("  Status:       {status_str}"));
        let started_str = format_datetime(&last_info.created_at);
        deps.ui.print(&format!("  Started:      {started_str}"));

        if let Some(completed) = &last_info.completed_at {
            let completed_str = format_datetime(completed);
            deps.ui.print(&format!("  Completed:    {completed_str}"));
        }

        if let Some(error) = &last_info.error {
            deps.ui
                .print_styled(&format!("  Error:        {error}"), MessageStyle::Red);
        }
    }

    deps.ui.print("");
}

fn display_app_status_json(
    app: &types::GetAppStatusResponseApp,
    deps: &Arc<AppDependencies>,
) -> Result<()> {
    let json = serde_json::to_string_pretty(app)?;
    deps.ui.print(&json);
    Ok(())
}

fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// App command arguments (matches CLI parser)
#[derive(Debug, Clone)]
pub struct AppArgs {
    /// Subcommand
    pub command: AppCommand,
}

/// Application subcommands
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// List all applications
    List {
        /// Output format
        format: OutputFormat,
    },
    /// Get status of an application
    Status {
        /// Application name
        app_name: String,
        /// Output format
        format: OutputFormat,
    },
    /// Delete an application
    Delete {
        /// Application name
        app_name: String,
        /// Force deletion without confirmation
        force: bool,
    },
}

impl OutputFormat {
    /// Parse output format from string
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            _ => anyhow::bail!("Invalid output format: {}", s),
        }
    }
}

/// Execute the app command with default dependencies
pub async fn execute(args: AppArgs) -> Result<()> {
    use ftl_common::RealUserInterface;
    use ftl_core::deps::{CredentialsProvider, RealCredentialsProvider, RealFtlApiClient};

    // Get credentials first to create authenticated API client
    let credentials_provider = RealCredentialsProvider;
    let credentials = credentials_provider.get_or_refresh_credentials().await?;

    let ui = Arc::new(RealUserInterface);
    let deps = Arc::new(AppDependencies {
        ui: ui.clone(),
        api_client: Arc::new(RealFtlApiClient::new_with_auth(
            ftl_core::api_client::Client::new(&ftl_core::config::DEFAULT_API_BASE_URL),
            credentials.access_token,
        )),
    });

    match args.command {
        AppCommand::List { format } => list_with_deps(format, &deps).await,
        AppCommand::Status { app_name, format } => status_with_deps(&app_name, format, &deps).await,
        AppCommand::Delete { app_name, force } => delete_with_deps(&app_name, force, &deps).await,
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
