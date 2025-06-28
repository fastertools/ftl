use std::{collections::HashMap, path::Path, process::Command};

use anyhow::{Context, Result};
use serde_json;

use crate::manifest::ToolManifest;

pub async fn execute(tool_path: String, verbose: bool) -> Result<()> {
    let tool_dir = Path::new(&tool_path);
    if !tool_dir.exists() {
        anyhow::bail!("Tool directory '{}' not found", tool_path);
    }

    let manifest_path = tool_dir.join("ftl.toml");
    if !manifest_path.exists() {
        anyhow::bail!("No ftl.toml found in '{}'", tool_path);
    }

    let manifest = ToolManifest::load(&manifest_path)?;
    let tool_name = &manifest.tool.name;
    let build_profile = &manifest.build.profile;

    println!("📊 Size analysis for tool: {tool_name}");
    println!("   Profile: {build_profile}");

    // Determine the profile directory based on manifest
    let profile_dir = match build_profile.as_str() {
        "dev" => "debug",
        _ => build_profile,
    };

    // Check if WASM file exists - convert hyphens to underscores for Cargo's naming
    // convention
    let wasm_filename = format!("{}.wasm", tool_name.replace('-', "_"));
    let wasm_path = tool_dir
        .join("target/wasm32-wasip1")
        .join(profile_dir)
        .join(&wasm_filename);

    if !wasm_path.exists() {
        println!("\n⚠️  WASM binary not found. Building first...");
        crate::commands::build::execute(Some(tool_path.clone()), None).await?;
    }

    // Get file metadata
    let metadata = std::fs::metadata(&wasm_path).context("Failed to read WASM file metadata")?;
    let wasm_size = metadata.len();

    // Get build time
    if let Ok(modified) = metadata.modified() {
        if let Ok(elapsed) = std::time::SystemTime::now().duration_since(modified) {
            let age = format_duration(elapsed.as_secs());
            println!("   Built: {age}");
        }
    }

    println!("\n📦 Binary Sizes:");
    println!("   WASM: {}", format_size(wasm_size));

    // Check for optimized version if using wasm-opt
    let opt_wasm_path = wasm_path.with_extension("opt.wasm");
    if opt_wasm_path.exists() {
        let opt_size = std::fs::metadata(&opt_wasm_path)?.len();
        println!(
            "   Optimized WASM: {} ({}% reduction)",
            format_size(opt_size),
            ((wasm_size - opt_size) * 100 / wasm_size)
        );
    }

    // Run wasm-tools if available to get detailed info
    if which::which("wasm-tools").is_ok() {
        println!("\n🔍 Detailed Analysis:");

        // Get module info
        let output = Command::new("wasm-tools")
            .arg("print")
            .arg(&wasm_path)
            .arg("--print-offsets")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                // Count sections
                let content = String::from_utf8_lossy(&output.stdout);
                let func_count = content.matches("(func ").count();
                let type_count = content.matches("(type ").count();
                let import_count = content.matches("(import ").count();
                let export_count = content.matches("(export ").count();

                println!("   Functions: {func_count}");
                println!("   Types: {type_count}");
                println!("   Imports: {import_count}");
                println!("   Exports: {export_count}");
            }
        }

        // Get section sizes
        let output = Command::new("wasm-tools")
            .arg("objdump")
            .arg(&wasm_path)
            .arg("--section-offsets")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                println!("\n📋 Section Breakdown:");

                // Parse section info
                let mut sections = Vec::new();
                for line in content.lines() {
                    if line.contains("section") && line.contains("size") {
                        sections.push(line.trim().to_string());
                    }
                }

                // Sort sections by size if possible
                sections.sort_by(|a, b| {
                    let size_a = extract_size_from_section(a).unwrap_or(0);
                    let size_b = extract_size_from_section(b).unwrap_or(0);
                    size_b.cmp(&size_a)
                });

                for section in sections.iter().take(if verbose { 20 } else { 5 }) {
                    println!("   {section}");
                }

                if verbose && sections.len() > 20 {
                    println!("   ... and {} more sections", sections.len() - 20);
                }
            }
        }

        // Verbose mode: Show detailed import analysis
        if verbose {
            println!("\n📥 Import Analysis (Startup Cost):");

            let output = Command::new("wasm-tools")
                .arg("print")
                .arg(&wasm_path)
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let content = String::from_utf8_lossy(&output.stdout);
                    let mut import_counts: HashMap<String, usize> = HashMap::new();

                    for line in content.lines() {
                        if line.contains("(import \"") {
                            if let Some(module) = extract_import_module(line) {
                                *import_counts.entry(module).or_insert(0) += 1;
                            }
                        }
                    }

                    let mut imports: Vec<_> = import_counts.into_iter().collect();
                    imports.sort_by(|a, b| b.1.cmp(&a.1));

                    for (module, count) in imports {
                        println!("   {module}: {count} imports");
                    }

                    println!("\n💡 Import Tips:");
                    println!("   - Each import adds startup overhead");
                    println!("   - Consider bundling multiple operations into single imports");
                    println!("   - Lazy-load optional functionality");
                }
            }

            // Show size history if available
            let size_history_path = tool_dir.join(".ftl").join("size_history.json");
            if size_history_path.exists() {
                println!("\n📈 Size History:");
                if let Ok(history) = std::fs::read_to_string(&size_history_path) {
                    if let Ok(history_data) = serde_json::from_str::<serde_json::Value>(&history) {
                        if let Some(entries) = history_data.as_array() {
                            let current_size_in_history = entries
                                .last()
                                .and_then(|e| e["size"].as_u64())
                                .unwrap_or(wasm_size);

                            // Show current if different from last recorded
                            if current_size_in_history != wasm_size {
                                println!(
                                    "   Current: {} ({})",
                                    format_size(wasm_size),
                                    if wasm_size > current_size_in_history {
                                        format!(
                                            "+{}",
                                            format_size(wasm_size - current_size_in_history)
                                        )
                                    } else {
                                        format!(
                                            "-{}",
                                            format_size(current_size_in_history - wasm_size)
                                        )
                                    }
                                );
                            }

                            // Show last 5 entries
                            for entry in entries.iter().rev().take(5) {
                                if let Some(size) = entry["size"].as_u64() {
                                    let date_display =
                                        if let Some(timestamp) = entry["timestamp"].as_u64() {
                                            format_timestamp(timestamp)
                                        } else {
                                            entry["date"].as_str().unwrap_or("Unknown").to_string()
                                        };

                                    println!(
                                        "   {}: {}",
                                        date_display,
                                        format_size(size)
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // Save current size to history
            save_size_to_history(tool_dir, wasm_size)?;
        }
    } else {
        println!("\n💡 Tip: Install wasm-tools for detailed binary analysis");
        println!("   cargo install wasm-tools");
    }

    // Size recommendations (only in verbose mode)
    if verbose {
        println!("\n💡 Optimization Suggestions:");

        let mut suggestions = Vec::new();

        if wasm_size > 5_000_000 {
            suggestions.push(
                "⚠️  Binary is larger than 5MB - optimization strongly recommended".to_string(),
            );
        }

        // Check optimization flags
        let has_opt_flags = manifest
            .optimization
            .flags
            .iter()
            .any(|f| f.contains("-O") || f.contains("--optimize"));

        if !has_opt_flags {
            suggestions.push(
                "Add wasm-opt optimization to ftl.toml:\n     [optimization]\n     flags =                  [\"-O3\", \"--enable-bulk-memory\"]"
                    .to_string(),
            );
        } else {
            // Check if they could use more aggressive optimization
            let has_o3_or_higher = manifest.optimization.flags.iter().any(|f| {
                f.contains("-O3") || f.contains("-O4") || f.contains("-Os") || f.contains("-Oz")
            });

            if !has_o3_or_higher {
                suggestions.push(format!(
                    "Consider more aggressive optimization in ftl.toml:\n     Current: {:?}\n     \
                     Try: [\"-O3\"] or [\"-Oz\"] for size",
                    manifest.optimization.flags
                ));
            }
        }

        // Profile suggestions
        match manifest.build.profile.as_str() {
            "dev" | "debug" => {
                suggestions.push(format!(
                    "Using '{}' profile - switch to 'release' for smaller binaries:\n     \
                     [build]\n     profile = \"release\"",
                    manifest.build.profile
                ));
            }
            "release" => {
                if wasm_size > 1_000_000 {
                    suggestions.push(
                        "Consider 'tiny' profile for maximum size reduction:\n     [build]\n     \
                         profile = \"tiny\""
                            .to_string(),
                    );
                }
            }
            _ => {}
        }

        // Cargo.toml suggestions
        suggestions.push(
            "Review Cargo.toml for optimization opportunities:\n     - Remove unused \
             dependencies\n     - Use default-features = false where possible\n     - Consider \
             lighter alternatives to heavy crates"
                .to_string(),
        );

        // Show suggestions
        for (i, suggestion) in suggestions.iter().enumerate() {
            if i > 0 {
                println!();
            }
            println!("   {suggestion}");
        }
    } // End of verbose mode optimization suggestions

    // Check dependencies size impact
    if let Ok(metadata) = cargo_metadata::MetadataCommand::new()
        .current_dir(tool_dir)
        .exec()
    {
        // Count direct dependencies vs total
        let total_packages = metadata.packages.len();
        let workspace_member = metadata.workspace_members.first();

        let direct_deps = if let Some(member_id) = workspace_member {
            metadata
                .packages
                .iter()
                .find(|p| &p.id == member_id)
                .map(|p| p.dependencies.len())
                .unwrap_or(0)
        } else {
            0
        };

        let transitive_deps = total_packages.saturating_sub(1); // Exclude the tool itself

        // In verbose mode, show full dependency analysis
        if verbose && transitive_deps > 20 {
            println!();
            println!("   📦 Dependency Analysis:");
            println!("      Direct dependencies: {direct_deps}");
            println!("      Total (including transitive): {transitive_deps}");

            if transitive_deps > 50 {
                println!("      ⚠️  High dependency count may significantly increase binary size");
                println!();
                println!("      To investigate dependencies:");
                println!("      • See which crates bring in the most dependencies:");
                println!("        cargo tree --duplicates");
                println!("      • Find a specific heavy dependency:");
                println!("        cargo tree -i <crate-name>");
                println!("      • See dependency tree sorted by depth:");
                println!("        cargo tree --depth 1");
            }
        }

        // Always show heaviest dependencies (in both normal and verbose modes)
        if let Some(member_id) = workspace_member {
            if let Some(package) = metadata.packages.iter().find(|p| &p.id == member_id) {
                let mut dep_weights: Vec<(String, usize)> = Vec::new();

                for dep in &package.dependencies {
                    // Count how many packages depend on this dependency
                    let weight = count_dependency_weight(&metadata, &dep.name);
                    dep_weights.push((dep.name.clone(), weight));
                }

                // Sort by weight
                dep_weights.sort_by(|a, b| b.1.cmp(&a.1));

                // Show top 5 if any are significant
                let significant_deps: Vec<_> = dep_weights
                    .iter()
                    .take(5)
                    .filter(|(_, weight)| *weight > 5)
                    .collect();

                if !significant_deps.is_empty() {
                    println!("\n💡 Heaviest dependencies:");
                    for (name, weight) in significant_deps {
                        println!("   • {name} (brings in ~{weight} crates)");
                    }
                }
            }
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

fn extract_size_from_section(section: &str) -> Option<u64> {
    // Extract size from section string like "code section: size 123456"
    if let Some(size_pos) = section.find("size ") {
        let size_str = &section[size_pos + 5..];
        let size_end = size_str
            .find(|c: char| !c.is_numeric())
            .unwrap_or(size_str.len());
        size_str[..size_end].parse().ok()
    } else {
        None
    }
}

fn extract_import_module(line: &str) -> Option<String> {
    // Extract module name from import line like: (import "wasi:io/streams@0.2.0"
    // ...)
    if let Some(start) = line.find("(import \"") {
        let rest = &line[start + 9..];
        if let Some(end) = rest.find('"') {
            let full_module = &rest[..end];
            // Extract just the module part before @version
            if let Some(at_pos) = full_module.find('@') {
                Some(full_module[..at_pos].to_string())
            } else {
                Some(full_module.to_string())
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        "just now".to_string()
    } else if seconds < 3600 {
        format!("{} minutes ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

fn count_dependency_weight(metadata: &cargo_metadata::Metadata, dep_name: &str) -> usize {
    // Simple heuristic: count packages that have this dependency in their name
    // This catches things like "serde" counting "serde_derive", "serde_json", etc.
    metadata
        .packages
        .iter()
        .filter(|p| p.name.contains(dep_name) || p.dependencies.iter().any(|d| d.name == dep_name))
        .count()
}

fn save_size_to_history(tool_dir: &Path, size: u64) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ftl_dir = tool_dir.join(".ftl");
    std::fs::create_dir_all(&ftl_dir)?;

    let history_path = ftl_dir.join("size_history.json");

    // Load existing history
    let mut history: Vec<serde_json::Value> = if history_path.exists() {
        let content = std::fs::read_to_string(&history_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Format date simply
    let date = format!("{timestamp}");

    // Add new entry
    history.push(serde_json::json!({
        "date": date,
        "size": size,
        "timestamp": timestamp,
    }));

    // Keep only last 20 entries
    if history.len() > 20 {
        let skip_count = history.len() - 20;
        history = history.into_iter().skip(skip_count).collect();
    }

    // Save updated history
    std::fs::write(&history_path, serde_json::to_string_pretty(&history)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }
}
