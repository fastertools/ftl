use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::task::JoinSet;

use crate::{
    common::config::FtlConfig,
    manifest::{ToolkitConfig, ToolkitManifest, ToolkitTool},
    spin_generator::SpinConfig,
};

pub async fn build(name: String, tools: Vec<String>) -> Result<()> {
    println!(
        "{} Building toolkit: {} with {} tools",
        style("→").cyan(),
        style(&name).bold(),
        tools.len()
    );

    if tools.is_empty() {
        anyhow::bail!("No tools specified for toolkit");
    }

    // Create toolkit directory
    let toolkit_dir = PathBuf::from(&name);
    std::fs::create_dir_all(&toolkit_dir)?;

    // First check that all tools exist
    for tool_name in &tools {
        let tool_dir = PathBuf::from(tool_name);
        if !tool_dir.join("ftl.toml").exists() {
            anyhow::bail!("Tool '{tool_name}' not found");
        }
    }

    // Build all tools in parallel
    println!();
    let tools_list = tools.join(", ");
    println!("Tools to build: {tools_list}");
    println!();

    // Create a single progress bar that shows overall progress
    let pb = ProgressBar::new(tools.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:30.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  ")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message("🔨 Building...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut tasks = JoinSet::new();
    let completed = Arc::new(std::sync::Mutex::new(Vec::new()));

    for tool_name in tools.clone() {
        let toolkit_dir = toolkit_dir.clone();
        let pb = pb.clone();
        let completed = completed.clone();
        let tools_count = tools.len();

        tasks.spawn(async move {
            // Build the tool quietly
            let build_result =
                crate::commands::build::execute_quiet(&tool_name, Some("release".to_string()))
                    .await;

            if let Err(e) = build_result {
                return Err(anyhow::anyhow!("Failed to build {tool_name}: {e}"));
            }

            // Update progress
            pb.inc(1);
            let mut comp = completed.lock().unwrap();
            comp.push(tool_name.clone());
            let done = comp.len();

            if done < tools_count {
                pb.set_message(format!("completed {tool_name}"));
            }

            // Find and copy WASM - check multiple locations
            let tool_dir = PathBuf::from(&tool_name);
            let wasm_filename = format!("{}.wasm", tool_name.replace('-', "_"));

            // Try Rust path first
            let rust_wasm_path = tool_dir
                .join("target")
                .join("wasm32-wasip1")
                .join("release")
                .join(&wasm_filename);

            // Try JavaScript path
            let js_wasm_filename = format!("{tool_name}.wasm");
            let js_wasm_path = tool_dir.join("dist").join(&js_wasm_filename);

            let (wasm_path, final_wasm_filename) = if rust_wasm_path.exists() {
                (rust_wasm_path, wasm_filename)
            } else if js_wasm_path.exists() {
                (js_wasm_path, js_wasm_filename)
            } else {
                return Err(anyhow::anyhow!(
                    "WASM binary not found for tool: {}",
                    tool_name
                ));
            };

            // Copy WASM to toolkit directory
            let dest_path = toolkit_dir.join(&final_wasm_filename);
            std::fs::copy(&wasm_path, &dest_path)?;

            // Path relative to .ftl/ directory
            Ok((tool_name, format!("../{final_wasm_filename}")))
        });
    }

    // Wait for all builds to complete
    let mut tool_paths = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(tool_info)) => tool_paths.push(tool_info),
            Ok(Err(e)) => {
                pb.abandon();
                return Err(e);
            }
            Err(e) => {
                pb.abandon();
                return Err(anyhow::anyhow!("Task failed: {e}"));
            }
        }
    }

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.green} [{bar:30.green}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.set_prefix("✓");
    pb.finish_with_message("✓ All tools built successfully");

    println!();
    println!("{} Building gateway component...", style("→").cyan());

    // Create toolkit manifest
    let tool_count = tools.len();
    let toolkit_manifest = ToolkitManifest {
        toolkit: ToolkitConfig {
            name: name.clone(),
            version: "1.0.0".to_string(),
            description: format!("Toolkit containing {tool_count} tools"),
        },
        tools: tools
            .iter()
            .map(|tool_name| ToolkitTool {
                name: tool_name.clone(),
                route: format!("/{tool_name}"),
            })
            .collect(),
        gateway: None, // Gateway is opt-in
    };

    // Save toolkit manifest
    let manifest_path = toolkit_dir.join("toolkit.toml");
    toolkit_manifest.save(&manifest_path)?;

    // Create .ftl directory in toolkit
    let ftl_dir = toolkit_dir.join(".ftl");
    std::fs::create_dir_all(&ftl_dir)?;

    // Generate gateway code
    generate_gateway_code(&toolkit_dir, &toolkit_manifest)?;

    // Build the gateway
    let gateway_output = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-wasip1",
            "--release",
            "--manifest-path",
            "gateway/Cargo.toml",
        ])
        .current_dir(&toolkit_dir)
        .output()
        .context("Failed to build gateway")?;

    if !gateway_output.status.success() {
        anyhow::bail!(
            "Gateway build failed: {}",
            String::from_utf8_lossy(&gateway_output.stderr)
        );
    }

    // Copy gateway WASM to toolkit directory
    let gateway_wasm_path = toolkit_dir
        .join("gateway")
        .join("target")
        .join("wasm32-wasip1")
        .join("release")
        .join(format!(
            "{}_gateway.wasm",
            toolkit_manifest.toolkit.name.replace('-', "_")
        ));

    let gateway_dest = toolkit_dir.join("gateway.wasm");
    std::fs::copy(&gateway_wasm_path, &gateway_dest).context("Failed to copy gateway WASM")?;

    println!("{} Gateway built successfully", style("✓").green());

    // Generate spin.toml for toolkit
    let spin_config = SpinConfig::from_toolkit(&toolkit_manifest, &tool_paths)?;
    let spin_path = ftl_dir.join("spin.toml");
    spin_config.save(&spin_path)?;

    println!();
    println!("{} Toolkit built successfully!", style("✓").green());
    println!();
    println!("Toolkit directory: {}", toolkit_dir.display());
    println!("Tools included:");
    for tool in &tools {
        println!("  - {tool}");
    }
    println!();
    println!("Next steps:");
    println!("  ftl toolkit serve {name}  # Serve locally");
    println!("  ftl toolkit deploy {name} # Deploy to FTL Edge");

    Ok(())
}

pub async fn serve(name: String, port: u16) -> Result<()> {
    println!(
        "{} Serving toolkit: {} on port {}",
        style("→").cyan(),
        style(&name).bold(),
        style(port).yellow()
    );

    let toolkit_dir = PathBuf::from(&name);
    if !toolkit_dir.exists() {
        anyhow::bail!(
            "Toolkit '{}' not found. Build it first with: ftl toolkit build",
            name
        );
    }

    // Check if spin.toml exists
    let spin_path = toolkit_dir.join(".ftl").join("spin.toml");
    if !spin_path.exists() {
        anyhow::bail!(".ftl/spin.toml not found in toolkit directory");
    }

    // Start spin server
    println!();
    println!("{} Starting toolkit server...", style("▶").green());
    println!();
    println!("  Toolkit: {name}");
    println!("  URL: http://localhost:{port}");

    // Load toolkit manifest to show available routes
    let manifest_path = toolkit_dir.join("toolkit.toml");
    if let Ok(manifest) = ToolkitManifest::load(&manifest_path) {
        println!("  Routes:");
        println!("    - {} (aggregates all tools)", style("/mcp").yellow());
        for tool in &manifest.tools {
            let route = &tool.route;
            println!("    - {route}/mcp");
        }
    }

    println!();
    println!("Press Ctrl+C to stop");
    println!();

    let mut child = Command::new("spin")
        .arg("up")
        .arg("--listen")
        .arg(format!("127.0.0.1:{port}"))
        .arg("--from")
        .arg(".ftl/spin.toml")
        .current_dir(&toolkit_dir)
        .spawn()
        .context("Failed to start spin server")?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!();
    println!("{} Stopping server...", style("■").red());

    // Kill the spin process
    child.kill().context("Failed to kill spin process")?;

    Ok(())
}

pub async fn deploy(name: String) -> Result<()> {
    println!(
        "{} Deploying toolkit: {}",
        style("→").cyan(),
        style(&name).bold()
    );

    let toolkit_dir = PathBuf::from(&name);
    if !toolkit_dir.exists() {
        anyhow::bail!(
            "Toolkit '{}' not found. Build it first with: ftl toolkit build",
            name
        );
    }

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
    let app_name = format!("{}{name}", config.get_app_prefix());

    // Try deploying without --create-name first (for existing apps)
    let output = Command::new("spin")
        .args(["aka", "deploy", "--from", ".ftl/spin.toml", "--no-confirm"])
        .current_dir(&toolkit_dir)
        .output()
        .context("Failed to run spin aka deploy")?;

    let output = if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If it fails because app doesn't exist, try with --create-name
        if stderr.contains("no app")
            || stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("No terminal available")
            || stderr.contains("must use --create-name")
        {
            spinner.set_message(format!("Creating new toolkit: {app_name}..."));
            Command::new("spin")
                .args([
                    "aka",
                    "deploy",
                    "--from",
                    ".ftl/spin.toml",
                    "--create-name",
                    &app_name,
                    "--no-confirm",
                ])
                .current_dir(&toolkit_dir)
                .output()
                .context("Failed to run spin aka deploy with --create-name")?
        } else {
            output
        }
    } else {
        output
    };

    spinner.finish_and_clear();

    if !output.status.success() {
        println!("{} Deployment failed", style("✗").red());
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{stderr}");
    }

    // Parse deployment URL
    let output_str = String::from_utf8_lossy(&output.stdout);
    let full_url = output_str
        .lines()
        .find(|line| line.contains("https://"))
        .and_then(|line| {
            line.split_whitespace()
                .find(|word| word.starts_with("https://"))
        })
        .unwrap_or("(URL not found in output)");

    // Extract base URL (remove any path components)
    let base_url = if let Some(domain_end) = full_url.find(".tech") {
        &full_url[..domain_end + 5] // Include ".tech"
    } else if let Some(first_slash) = full_url[8..].find('/') {
        // Skip "https://"
        &full_url[..8 + first_slash]
    } else {
        full_url
    };

    println!();
    println!("{} Toolkit deployed successfully!", style("✓").green());
    println!();
    println!("Toolkit: {name}");
    println!("URL: {base_url}");

    // Show available routes
    let manifest_path = toolkit_dir.join("toolkit.toml");
    if let Ok(manifest) = ToolkitManifest::load(&manifest_path) {
        println!("Available endpoints:");
        println!("  - {base_url}/mcp (aggregates all tools)");
        for tool in &manifest.tools {
            let route = &tool.route;
            println!("  - {base_url}{route}/mcp");
        }
    }

    Ok(())
}

/// Generate the gateway code for a toolkit
fn generate_gateway_code(toolkit_dir: &Path, manifest: &ToolkitManifest) -> Result<()> {
    // Create gateway directory
    let gateway_dir = toolkit_dir.join("gateway");
    std::fs::create_dir_all(&gateway_dir)?;

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}-gateway"
version = "0.1.0"
edition = "2021"

[dependencies]
# TODO: Add gateway dependencies
spin-sdk = "3.1.1"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

[lib]
crate-type = ["cdylib"]

[workspace]"#,
        manifest.toolkit.name
    );
    std::fs::write(gateway_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    let src_dir = gateway_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Generate lib.rs with gateway configuration
    let lib_rs = format!(
        r#"use ftl_sdk_rs::{{ftl_mcp_gateway, gateway::{{GatewayConfig, ToolEndpoint}}, mcp::ServerInfo}};

// Configure the gateway with all tools in the toolkit
fn create_gateway_config() -> GatewayConfig {{
    GatewayConfig {{
        tools: vec![
{}
        ],
        server_info: ServerInfo {{
            name: "{}-gateway".to_string(),
            version: "{}".to_string(),
        }},
        // Base URL is empty - gateway uses component IDs directly
        base_url: "".to_string(),
    }}
}}

// Create the gateway component
ftl_mcp_gateway!(create_gateway_config());"#,
        manifest
            .tools
            .iter()
            .map(|tool| {
                format!(
                    r#"            ToolEndpoint {{
                name: "{}".to_string(),
                route: "{}".to_string(),
                description: None,
            }},"#,
                    tool.name, tool.route
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        manifest.toolkit.name,
        manifest.toolkit.version
    );
    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

    Ok(())
}
