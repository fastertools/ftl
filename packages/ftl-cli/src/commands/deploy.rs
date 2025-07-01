use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::common::{
    config::FtlConfig,
    manifest_utils::load_manifest_and_name,
    spin_installer::check_and_install_spin,
    spin_utils::{check_akamai_auth, deploy_to_akamai},
    tool_paths::{get_spin_toml_path, validate_tool_exists},
};

pub async fn execute(tool_path: String) -> Result<()> {
    println!(
        "{} Deploying tool: {}",
        style("→").cyan(),
        style(&tool_path).bold()
    );

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

    // Get spin path and check Akamai authentication
    let spin_path = check_and_install_spin().await?;
    check_akamai_auth(&spin_path).await?;

    // Deploy using spin aka with spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message("Deploying to FTL Edge...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    // Load config and generate app name with user prefix
    let config = FtlConfig::load().unwrap_or_default();
    let prefix = config.get_app_prefix();
    let app_name = format!("{prefix}{tool_name}");

    // Deploy with the generated app name
    let deployment_result = deploy_to_akamai(&tool_path, Some(&app_name)).await;

    // Handle deployment result
    match deployment_result {
        Ok(deployment_info) => {
            spinner.finish_and_clear();
            // Ensure URL includes /mcp path
            let full_url = if deployment_info.url.ends_with("/mcp") {
                deployment_info.url.clone()
            } else {
                let url = deployment_info.url.trim_end_matches('/');
                format!("{url}/mcp")
            };

            println!("{} Deployment successful!", style("✓").green());
            println!("  Name: {}", style(&deployment_info.app_name).cyan());
            println!("  URL: {}", style(&full_url).yellow().bold());
            println!();
            println!("Test your tool:");
            println!("  curl -X POST {full_url} \\");
            println!("    -H \"Content-Type: application/json\" \\");
            println!("    -d '{{\"jsonrpc\":\"2.0\",\"method\":\"tools/list\",\"id\":1}}'");
            println!();
            println!("Manage your deployment:");
            let app_name = &deployment_info.app_name;
            println!("  ftl status {app_name}");
            println!("  ftl logs {app_name}");
            println!("  ftl delete {app_name}");
            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            println!("{} Deployment failed", style("✗").red());
            anyhow::bail!("{e}");
        }
    }
}
