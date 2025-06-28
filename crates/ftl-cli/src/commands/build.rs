use std::{path::PathBuf, process::Command};

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    common::{
        build_utils::{format_size, get_file_size, optimize_wasm},
        manifest_utils::load_manifest_and_name,
        spin_utils::check_spin_installed,
        tool_paths::{ensure_ftl_dir, get_profile_dir, get_spin_toml_path, get_wasm_path},
    },
    spin_generator::SpinConfig,
};

pub async fn execute(name: Option<String>, profile: Option<String>) -> Result<()> {
    let tool_path = name.unwrap_or_else(|| ".".to_string());
    build_tool(&tool_path, profile, false).await
}

pub async fn execute_quiet(tool_path: &str, profile: Option<String>) -> Result<()> {
    build_tool(tool_path, profile, true).await
}

pub async fn execute_and_serve(name: Option<String>, profile: Option<String>) -> Result<()> {
    let tool_path = name.unwrap_or_else(|| ".".to_string());

    // First build the tool
    build_tool(&tool_path, profile, false).await?;

    // Then serve it
    println!();
    crate::commands::serve::execute(tool_path, 3000, false).await
}

async fn build_tool(tool_path: &str, profile: Option<String>, quiet: bool) -> Result<()> {
    if !quiet {
        println!(
            "{} Building tool: {}",
            style("→").cyan(),
            style(tool_path).bold()
        );
    }

    // Validate tool directory exists
    crate::common::tool_paths::validate_tool_exists(tool_path)?;

    // Load and validate manifest
    let (manifest, tool_name) = load_manifest_and_name(tool_path)?;
    manifest.validate()?;

    // Determine build profile
    let build_profile = profile.unwrap_or_else(|| manifest.build.profile.clone());

    // Ensure .ftl directory exists and check spin is installed
    ensure_ftl_dir(tool_path)?;
    check_spin_installed()?;

    // Generate spin.toml with build configuration
    let profile_dir = get_profile_dir(&build_profile);
    let wasm_filename = format!("{}.wasm", tool_name.replace('-', "_"));
    let relative_wasm_path = PathBuf::from("..")
        .join("target")
        .join("wasm32-wasip1")
        .join(profile_dir)
        .join(&wasm_filename);

    let spin_config = SpinConfig::from_tool(&manifest, &relative_wasm_path)?;
    let spin_path = get_spin_toml_path(tool_path);
    spin_config.save(&spin_path)?;

    // Create progress bar (only if not quiet)
    let pb = if quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(3);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb
    };

    // Step 1: Run spin build
    pb.set_message("Running spin build...");
    pb.inc(1);

    let spin_output = Command::new("spin")
        .arg("build")
        .arg("-f")
        .arg(".ftl/spin.toml")
        .current_dir(tool_path)
        .output()?;

    if !spin_output.status.success() {
        let stderr = String::from_utf8_lossy(&spin_output.stderr);
        anyhow::bail!("Spin build failed:\n{}", stderr);
    }

    // Step 2: Verify WASM was built
    pb.set_message("Verifying build output...");
    pb.inc(1);

    let wasm_path = get_wasm_path(tool_path, &tool_name, &build_profile);
    if !wasm_path.exists() {
        anyhow::bail!("WASM binary not found at: {}", wasm_path.display());
    }

    // Step 3: Run wasm-opt (post-build optimization)
    pb.set_message("Optimizing WASM binary...");
    pb.inc(1);

    // Always include flags to match Rust's target features
    let mut wasm_opt_flags = vec![
        "--enable-simd".to_string(),
        "--enable-bulk-memory".to_string(),
        "--enable-mutable-globals".to_string(),
        "--enable-sign-ext".to_string(),
        "--enable-nontrapping-float-to-int".to_string(),
        "--enable-reference-types".to_string(),
    ];

    // Add user-specified flags
    if !manifest.optimization.flags.is_empty() {
        wasm_opt_flags.extend(manifest.optimization.flags.clone());
    } else {
        // Default optimization if none specified
        wasm_opt_flags.push("-O2".to_string());
    }

    optimize_wasm(&wasm_path, &wasm_opt_flags)?;

    pb.finish_with_message("Build complete!");

    // Display binary size (only if not quiet)
    if !quiet {
        let size = get_file_size(&wasm_path)?;

        println!();
        println!("{} Build successful!", style("✓").green());
        println!("  Binary: {}", wasm_path.display());
        println!("  Size: {}", format_size(size));
        println!("  Profile: {build_profile}");
    }

    Ok(())
}
