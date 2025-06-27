use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::common::{
    config::FtlConfig,
    manifest_utils::load_manifest_and_name,
    spin_utils::{check_akamai_auth, deploy_to_akamai},
    tool_paths::{get_spin_toml_path, validate_tool_exists},
};

pub async fn execute(tool_path: String) -> Result<()> {
    println!("{} Deploying tool: {}", style("→").cyan(), style(&tool_path).bold());

    // Validate tool exists and load manifest
    validate_tool_exists(&tool_path)?;
    let (_manifest, tool_name) = load_manifest_and_name(&tool_path)?;

    // Ensure tool is built with production profile
    println!("{} Building release version...", style("→").cyan());
    crate::commands::build::execute(Some(tool_path.clone()), Some("release".to_string())).await?;

    // Check if spin.toml exists
    let spin_path = get_spin_toml_path(&tool_path);
    if !spin_path.exists() {
        anyhow::bail!(".ftl/spin.toml not found. This should have been created during build.");
    }

    // Check Akamai authentication
    check_akamai_auth()?;

    // Deploy using spin aka with spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    spinner.set_message("Deploying to FTL Edge...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    
    // Load config and generate app name with user prefix
    let config = FtlConfig::load().unwrap_or_default();
    let app_name = format!("{}{}", config.get_app_prefix(), tool_name);
    
    // Deploy with the generated app name in a separate thread
    let tool_path_clone = tool_path.clone();
    let app_name_clone = app_name.clone();
    let deployment_result = tokio::task::spawn_blocking(move || {
        deploy_to_akamai(&tool_path_clone, Some(&app_name_clone))
    });
    
    // Wait for deployment to complete
    match deployment_result.await.unwrap() {
        Ok(deployment_info) => {
            spinner.finish_and_clear();
            println!("{} Deployment successful!", style("✓").green());
            println!("  Name: {}", style(&app_name).cyan());
            println!("  URL: {}", style(&deployment_info.url).yellow().bold());
            println!();
            println!("Test your tool:");
            println!("  curl -X POST {} \\", deployment_info.url);
            println!("    -H \"Content-Type: application/json\" \\");
            println!("    -d '{{\"jsonrpc\":\"2.0\",\"method\":\"tools/list\",\"id\":1}}'");
            println!();
            println!("Manage your deployment:");
            println!("  ftl status {}", app_name);
            println!("  ftl logs {}", app_name);
            println!("  ftl delete {}", app_name);
            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            println!("{} Deployment failed", style("✗").red());
            anyhow::bail!("{}", e);
        }
    }
}