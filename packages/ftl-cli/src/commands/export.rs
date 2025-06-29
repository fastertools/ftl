use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use console::style;

use crate::common::{
    manifest_utils::load_manifest_and_name,
    tool_paths::{self, get_profile_dir, validate_tool_exists},
};

const WASI_ADAPTER_URL: &str = "https://github.com/bytecodealliance/wasmtime/releases/download/v22.0.0/wasi_snapshot_preview1.reactor.wasm";

pub async fn execute(
    name: Option<String>,
    output: Option<PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    let tool_path = name.unwrap_or_else(|| ".".to_string());

    println!(
        "{} Exporting tool: {}",
        style("→").cyan(),
        style(&tool_path).bold()
    );

    // Validate tool directory exists
    validate_tool_exists(&tool_path)?;

    // Load manifest to get tool name
    let (manifest, tool_name) = load_manifest_and_name(&tool_path)?;

    // Determine build profile
    let build_profile = profile.unwrap_or_else(|| manifest.build.profile.clone());
    let language = manifest.tool.language;

    // Get the WASM path based on language
    let wasm_path =
        tool_paths::get_wasm_path_for_language(&tool_path, &tool_name, &build_profile, language);

    if !wasm_path.exists() {
        anyhow::bail!(
            "WASM file not found at {}. Please run 'ftl build' first.",
            wasm_path.display()
        );
    }

    // Check if wasm-tools is installed
    if which::which("wasm-tools").is_err() {
        anyhow::bail!(
            "wasm-tools CLI not found. Please install it from: https://github.com/bytecodealliance/wasm-tools"
        );
    }

    // Determine output path
    let output_path = match output {
        Some(path) => path,
        None => {
            use crate::language::Language;
            match language {
                Language::Rust => {
                    let profile_dir = get_profile_dir(&build_profile);
                    PathBuf::from(&tool_path)
                        .join("target")
                        .join("wasm32-wasip1")
                        .join(profile_dir)
                        .join(format!("{}.component.wasm", tool_name.replace('-', "_")))
                }
                Language::JavaScript => {
                    // For JavaScript, put the component next to the WASM file in dist
                    PathBuf::from(&tool_path)
                        .join("dist")
                        .join(format!("{tool_name}.component.wasm"))
                }
            }
        }
    };

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check if the WASM file is already a component
    let validate_output = Command::new("wasm-tools")
        .args([
            "validate",
            wasm_path.to_str().unwrap(),
            "--features",
            "component-model",
        ])
        .output()
        .context("Failed to run wasm-tools validate")?;

    let is_already_component = validate_output.status.success();

    if is_already_component {
        // For JavaScript, the WASM is already a component, just copy it
        println!(
            "{} WASM is already a component, copying...",
            style("→").cyan()
        );
        fs::copy(&wasm_path, &output_path).context("Failed to copy component")?;
    } else {
        // For Rust, we need to componentize the module
        // Download WASI adapter if not already cached
        let adapter_path = get_wasi_adapter_path()?;
        if !adapter_path.exists() {
            println!("{} Downloading WASI adapter...", style("→").cyan());
            download_wasi_adapter(&adapter_path).await?;
        }

        // Run wasm-tools component new
        println!("{} Creating WASM component...", style("→").cyan());

        let output = Command::new("wasm-tools")
            .args([
                "component",
                "new",
                wasm_path.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
                "--adapt",
                adapter_path.to_str().unwrap(),
            ])
            .output()
            .context("Failed to run wasm-tools")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create WASM component:\n{}", stderr);
        }
    }

    let component_size = fs::metadata(&output_path)?.len();

    println!();
    println!("{} Export successful!", style("✓").green());
    println!("  Component: {}", output_path.display());
    println!("  Size: {}", format_file_size(component_size));
    println!();
    println!("You can now serve this component with:");
    println!("  wasmtime serve -Scli {}", output_path.display());

    Ok(())
}

fn get_wasi_adapter_path() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
        .join("ftl");

    fs::create_dir_all(&cache_dir)?;

    Ok(cache_dir.join("wasi_snapshot_preview1.reactor.wasm"))
}

async fn download_wasi_adapter(path: &Path) -> Result<()> {
    let response = reqwest::get(WASI_ADAPTER_URL)
        .await
        .context("Failed to download WASI adapter")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download WASI adapter: HTTP {}",
            response.status()
        );
    }

    let bytes = response.bytes().await?;
    fs::write(path, bytes).context("Failed to write WASI adapter")?;

    Ok(())
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
