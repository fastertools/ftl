use std::path::PathBuf;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    common::{
        build_utils::{format_size, get_file_size, optimize_wasm},
        manifest_utils::load_manifest_and_name,
        spin_installer::check_and_install_spin,
        tool_paths::{self, ensure_ftl_dir, get_spin_toml_path},
    },
    language::{Language, get_language_support},
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

    // Get language support
    let language_support = get_language_support(manifest.tool.language);

    // Ensure .ftl directory exists and check spin is installed
    ensure_ftl_dir(tool_path)?;
    check_and_install_spin().await?;

    // Copy spin.toml to .ftl directory if it doesn't exist
    let spin_toml_src = PathBuf::from(tool_path).join("spin.toml");
    let spin_toml_dest = get_spin_toml_path(tool_path);
    if spin_toml_src.exists() && !spin_toml_dest.exists() {
        std::fs::copy(&spin_toml_src, &spin_toml_dest)?;
    }

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

    // Step 1: Run language-specific build
    let language = manifest.tool.language;
    pb.set_message(format!("Building {language} tool..."));
    pb.inc(1);

    // Validate language environment
    language_support.validate_environment()?;

    // Run build commands from ftl.toml
    if let Some(commands) = &manifest.build.commands {
        for command in commands {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            let output = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .current_dir(tool_path)
                .output()?;
                
            if !output.status.success() {
                anyhow::bail!(
                    "Build command failed: {}\n{}",
                    command,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    } else {
        // Fallback to language-specific build
        language_support.build(&manifest, std::path::Path::new(tool_path))?;
    }

    // Step 2: Verify WASM was built
    pb.set_message("Verifying build output...");
    pb.inc(1);

    // Check for handler.wasm first (new structure)
    let wasm_path = PathBuf::from(tool_path).join("handler.wasm");
    let wasm_path = if wasm_path.exists() {
        wasm_path
    } else {
        // Fallback to old structure
        tool_paths::get_wasm_path_for_language(
            tool_path,
            &tool_name,
            &build_profile,
            manifest.tool.language,
        )
    };

    if !wasm_path.exists() {
        anyhow::bail!("WASM binary not found at: {}", wasm_path.display());
    }

    // Copy WASM to .ftl/dist directory for deployment
    let ftl_dist_dir = PathBuf::from(tool_path).join(".ftl").join("dist");
    std::fs::create_dir_all(&ftl_dist_dir)?;
    let dest_wasm = ftl_dist_dir.join("handler.wasm");
    if wasm_path != dest_wasm {
        std::fs::copy(&wasm_path, &dest_wasm)?;
    }

    // Step 3: Run wasm-opt (post-build optimization) - only for Rust
    if manifest.tool.language == Language::Rust {
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
    } else {
        pb.inc(1);
    }

    pb.finish_with_message("Build complete!");

    // Display binary size (only if not quiet)
    if !quiet {
        let size = get_file_size(&wasm_path)?;

        println!();
        println!("{} Build successful!", style("✓").green());
        println!("  Binary: {}", wasm_path.display());
        let size = format_size(size);
        println!("  Size: {size}");
        println!("  Profile: {build_profile}");
    }

    Ok(())
}
