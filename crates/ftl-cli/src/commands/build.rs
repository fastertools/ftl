use std::path::PathBuf;

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    common::{
        build_utils::{format_size, get_file_size, optimize_wasm},
        manifest_utils::load_manifest_and_name,
        spin_utils::check_spin_installed,
        tool_paths::{ensure_ftl_dir, get_profile_dir, get_spin_toml_path, get_wasm_path},
    },
    language::{get_language_support, Language},
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
    
    // Get language support
    let language_support = get_language_support(manifest.tool.language);

    // Ensure .ftl directory exists and check spin is installed
    ensure_ftl_dir(tool_path)?;
    check_spin_installed()?;

    // Generate spin.toml in .ftl directory
    match manifest.tool.language {
        Language::Rust => {
            // Generate spin.toml with build configuration for Rust
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
        }
        Language::JavaScript => {
            // For JavaScript, copy the existing spin.toml to .ftl/spin.toml
            let source_spin = PathBuf::from(tool_path).join("spin.toml");
            let dest_spin = get_spin_toml_path(tool_path);
            
            if source_spin.exists() {
                std::fs::copy(&source_spin, &dest_spin)
                    .context("Failed to copy spin.toml to .ftl directory")?;
            } else {
                // If no spin.toml exists, generate one
                let relative_wasm_path = PathBuf::from("..")
                    .join("dist")
                    .join(format!("{}.wasm", tool_name));

                let spin_config = SpinConfig::from_tool(&manifest, &relative_wasm_path)?;
                spin_config.save(&dest_spin)?;
            }
        }
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
    pb.set_message(format!("Building {} tool...", manifest.tool.language));
    pb.inc(1);

    // Validate language environment
    language_support.validate_environment()?;
    
    // Run the language-specific build
    language_support.build(&manifest, std::path::Path::new(tool_path))?;

    // Step 2: Verify WASM was built
    pb.set_message("Verifying build output...");
    pb.inc(1);

    let wasm_path = match manifest.tool.language {
        Language::Rust => get_wasm_path(tool_path, &tool_name, &build_profile),
        Language::JavaScript => {
            // For JS/TS, Spin puts the WASM in dist/{tool-name}.wasm
            PathBuf::from(tool_path).join("dist").join(format!("{}.wasm", tool_name))
        }
    };
    
    if !wasm_path.exists() {
        anyhow::bail!("WASM binary not found at: {}", wasm_path.display());
    }

    // For JavaScript, also copy the WASM to .ftl/dist directory for deployment
    if let Language::JavaScript = manifest.tool.language {
        let ftl_dist_dir = PathBuf::from(tool_path).join(".ftl").join("dist");
        std::fs::create_dir_all(&ftl_dist_dir)?;
        let dest_wasm = ftl_dist_dir.join(format!("{}.wasm", tool_name));
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
        println!("  Size: {}", format_size(size));
        println!("  Profile: {build_profile}");
    }

    Ok(())
}
