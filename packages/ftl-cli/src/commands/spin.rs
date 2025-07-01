use anyhow::{Context, Result};
use console::style;
use dialoguer::Confirm;
use std::process::Command;

use crate::common::spin_installer::{SPIN_REQUIRED_VERSION, check_and_install_spin};

pub async fn install() -> Result<()> {
    println!(
        "{} Installing Spin v{}",
        style("→").cyan(),
        SPIN_REQUIRED_VERSION
    );

    check_and_install_spin().await?;

    Ok(())
}

pub async fn update() -> Result<()> {
    println!(
        "{} Updating Spin to v{}",
        style("→").cyan(),
        SPIN_REQUIRED_VERSION
    );

    // For now, update is the same as install since the script handles it
    check_and_install_spin().await?;

    Ok(())
}

pub async fn remove() -> Result<()> {
    println!("{} Removing Spin", style("→").cyan());

    // Check FTL-managed installation first
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let ftl_bin_dir = home_dir.join(".ftl").join("bin");
    let ftl_spin_path = ftl_bin_dir.join("spin");

    if ftl_spin_path.exists() {
        let confirm = Confirm::new()
            .with_prompt("Remove FTL-managed Spin installation?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("Removal cancelled");
            return Ok(());
        }

        std::fs::remove_file(&ftl_spin_path)?;
        println!(
            "{} FTL-managed Spin removed successfully",
            style("✓").green()
        );
        return Ok(());
    }

    // Check if spin is in PATH (system-wide)
    if let Ok(spin_path) = which::which("spin") {
        println!("Found system-wide Spin at: {}", spin_path.display());
        println!("⚠️  This appears to be a system-wide installation.");
        println!("FTL cannot remove system-wide installations.");

        // Try to provide OS-specific instructions
        match std::env::consts::OS {
            "macos" => {
                println!("\nFor macOS, you might have installed it via:");
                println!("  - Homebrew: brew uninstall fermyon-spin");
                println!("  - Or manually via the install script");
            }
            "linux" => {
                println!("\nFor Linux, check if you installed it via:");
                println!("  - Your package manager (apt, yum, dnf, etc.)");
                println!("  - Or manually via the install script");
            }
            _ => {}
        }
    } else {
        println!("Spin is not installed");
    }

    Ok(())
}

pub async fn info() -> Result<()> {
    println!("{} Spin Installation Info", style("ℹ").blue());
    println!();

    // Check FTL-managed installation first
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let ftl_bin_dir = home_dir.join(".ftl").join("bin");
    let ftl_spin_path = ftl_bin_dir.join("spin");

    let mut found_ftl_managed = false;
    if ftl_spin_path.exists() {
        found_ftl_managed = true;
        println!("FTL-managed Spin:");
        println!("  Path: {}", ftl_spin_path.display());

        // Get version
        let output = Command::new(&ftl_spin_path).arg("--version").output()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Version: {}", version.trim());
        }
        println!();
    }

    // Check system-wide installation
    if let Ok(system_spin_path) = which::which("spin") {
        // Skip if it's the same as FTL-managed
        if !found_ftl_managed || system_spin_path != ftl_spin_path {
            println!("System-wide Spin:");
            println!("  Path: {}", system_spin_path.display());

            // Get version
            let output = Command::new(&system_spin_path).arg("--version").output()?;

            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("  Version: {}", version.trim());
            }
            println!();
        }
    }

    println!("FTL requires Spin v{}", SPIN_REQUIRED_VERSION);
    println!();

    if found_ftl_managed {
        println!("✓ FTL will use the managed installation in ~/.ftl/bin");
    } else if which::which("spin").is_ok() {
        println!("⚠️  System Spin found, but FTL prefers its own managed version");
        println!(
            "   Run 'ftl spin install' to install Spin v{} in ~/.ftl/bin",
            SPIN_REQUIRED_VERSION
        );
        println!("   This ensures version compatibility and won't affect your system installation");
    } else {
        println!("❌ Spin is not installed");
        println!(
            "   Run 'ftl spin install' to install Spin v{} in ~/.ftl/bin",
            SPIN_REQUIRED_VERSION
        );
    }

    Ok(())
}
