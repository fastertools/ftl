use anyhow::{Context, Result};
use console::style;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

pub const SPIN_REQUIRED_VERSION: &str = "3.3.1";
const SPIN_RELEASES_URL: &str = "https://github.com/fermyon/spin/releases/download";

/// Get the path to spin if it exists (does not install)
pub fn get_spin_path() -> Result<PathBuf> {
    // First check if FTL-managed Spin is installed in ~/.ftl/bin
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let ftl_bin_dir = home_dir.join(".ftl").join("bin");
    let spin_path = ftl_bin_dir.join("spin");

    if spin_path.exists() {
        return Ok(spin_path);
    }

    // If no FTL-managed version, check if spin is available in PATH
    if let Ok(system_spin_path) = which::which("spin") {
        return Ok(system_spin_path);
    }

    anyhow::bail!("Spin not found")
}

pub async fn check_and_install_spin() -> Result<PathBuf> {
    // First check if FTL-managed Spin is installed in ~/.ftl/bin
    let home_dir = dirs::home_dir().context("Could not determine home directory")?;
    let ftl_bin_dir = home_dir.join(".ftl").join("bin");
    let spin_path = ftl_bin_dir.join("spin");

    if spin_path.exists() {
        debug!("Found FTL-managed Spin at: {:?}", spin_path);
        ensure_akamai_plugin(&spin_path)?;
        return Ok(spin_path);
    }

    // If no FTL-managed version, check if spin is available in PATH
    if let Ok(system_spin_path) = which::which("spin") {
        debug!("Found system Spin in PATH at: {:?}", system_spin_path);

        // Check if system spin version is compatible
        if let Ok(version) = get_spin_version(&system_spin_path) {
            if is_version_compatible(&version, SPIN_REQUIRED_VERSION)? {
                debug!(
                    "System Spin version {} is compatible with required version {}",
                    version, SPIN_REQUIRED_VERSION
                );
                ensure_akamai_plugin(&system_spin_path)?;
                return Ok(system_spin_path);
            } else {
                info!(
                    "System Spin version {} is older than required version {}",
                    version, SPIN_REQUIRED_VERSION
                );
            }
        }
    }

    // Need to install
    let auto_install = env::var("FTL_AUTO_INSTALL").unwrap_or_default() == "true";

    if !auto_install {
        eprintln!("⚠️  FTL requires Spin v{SPIN_REQUIRED_VERSION} to run WebAssembly tools.");
        eprintln!("This will be installed in ~/.ftl/bin (not system-wide).");

        if which::which("spin").is_ok() {
            eprintln!();
            eprintln!("Note: System Spin detected, but it's not compatible with FTL requirements.");
            eprintln!(
                "FTL will install its own version. Your system installation won't be affected."
            );
        }

        // Check if we're in a terminal
        if atty::is(atty::Stream::Stdin) {
            let should_install = Confirm::new()
                .with_prompt("Would you like to install Spin now?")
                .default(true)
                .interact()?;

            if !should_install {
                anyhow::bail!("Spin installation is required to continue");
            }
        } else {
            // Non-interactive mode (CI, scripts, etc)
            eprintln!("Running in non-interactive mode, proceeding with installation...");
        }
    }

    install_spin(&ftl_bin_dir, &spin_path).await?;

    // Verify installation
    if spin_path.exists() {
        ensure_akamai_plugin(&spin_path)?;
        Ok(spin_path)
    } else {
        anyhow::bail!("Failed to install Spin")
    }
}

async fn install_spin(bin_dir: &PathBuf, spin_path: &PathBuf) -> Result<()> {
    info!("Installing Spin v{SPIN_REQUIRED_VERSION}");

    // Create bin directory
    fs::create_dir_all(bin_dir).context("Failed to create FTL bin directory")?;

    let download_url = get_download_url()?;
    eprintln!("📥 Downloading Spin...");

    let (temp_file, _temp_dir) = download_spin(&download_url).await?;
    debug!("Downloaded archive to: {:?}", temp_file);
    extract_and_install(&temp_file, spin_path)?;

    println!("{} Spin installed successfully!", style("✓").green());

    // Install the Akamai plugin
    eprintln!("📦 Installing Akamai plugin...");

    // Verify spin binary exists and is executable
    if !spin_path.exists() {
        anyhow::bail!("Spin binary not found at {spin_path:?} after extraction");
    }

    let plugin_output = Command::new(spin_path)
        .args(["plugin", "install", "aka"])
        .output()
        .with_context(|| format!("Failed to run spin at {spin_path:?}"))?;

    if !plugin_output.status.success() {
        let stderr = String::from_utf8_lossy(&plugin_output.stderr);
        eprintln!("⚠️  Warning: Failed to install Akamai plugin: {stderr}");
        eprintln!("   You can install it manually with: spin plugin install aka");
    } else {
        println!(
            "{} Akamai plugin installed successfully!",
            style("✓").green()
        );
    }

    Ok(())
}

fn get_download_url() -> Result<String> {
    let platform = match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => "linux-amd64",
        ("linux", "aarch64") => "linux-aarch64",
        ("macos", "x86_64") => "macos-amd64",
        ("macos", "aarch64") => "macos-aarch64",
        ("windows", "x86_64") => "windows-amd64",
        _ => {
            let os = env::consts::OS;
            let arch = env::consts::ARCH;
            anyhow::bail!("Unsupported platform: {os} {arch}")
        }
    };

    let extension = if env::consts::OS == "windows" {
        "zip"
    } else {
        "tar.gz"
    };

    Ok(format!(
        "{SPIN_RELEASES_URL}/v{SPIN_REQUIRED_VERSION}/spin-v{SPIN_REQUIRED_VERSION}-{platform}.{extension}"
    ))
}

async fn download_spin(url: &str) -> Result<(PathBuf, tempfile::TempDir)> {
    debug!("Downloading from: {}", url);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to download from {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Download failed with status: {} for URL: {}",
            response.status(),
            url
        );
    }

    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("Failed to get download size"))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} {eta}")?
            .progress_chars("#>-"),
    );
    pb.set_message("Downloading");

    let temp_dir = tempfile::Builder::new()
        .prefix("ftl-spin-")
        .tempdir()
        .context("Failed to create temp directory")?;
    let temp_file = temp_dir.path().join("spin-archive");
    let mut file = fs::File::create(&temp_file).context("Failed to create temp file")?;

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to download chunk")?;
        file.write_all(&chunk)?;
        file.flush()?; // Flush after each chunk
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    // Ensure all data is written
    file.flush().context("Failed to flush file")?;
    file.sync_all().context("Failed to sync file to disk")?;
    drop(file); // Close the file handle

    pb.finish_with_message("Download complete");
    debug!("Download complete, file size: {} bytes", downloaded);
    Ok((temp_file, temp_dir))
}

fn extract_and_install(archive_path: &PathBuf, target_path: &PathBuf) -> Result<()> {
    // Verify the archive exists
    if !archive_path.exists() {
        anyhow::bail!("Archive file does not exist at {:?}", archive_path);
    }

    debug!(
        "Archive file size: {} bytes",
        fs::metadata(archive_path)?.len()
    );

    if cfg!(windows) {
        extract_zip(archive_path, target_path)?;
    } else {
        extract_tar_gz(archive_path, target_path)?;
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(target_path, perms)?;
    }

    Ok(())
}

fn extract_tar_gz(archive_path: &PathBuf, target_path: &PathBuf) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    debug!(
        "Extracting tar.gz from {:?} to {:?}",
        archive_path, target_path
    );

    let file = fs::File::open(archive_path)
        .with_context(|| format!("Failed to open archive file at {archive_path:?}"))?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    // Create parent directory if it doesn't exist
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).context("Failed to create parent directory")?;
    }

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        debug!("Archive entry: {:?}", path);

        if path.file_name() == Some(std::ffi::OsStr::new("spin")) {
            debug!("Found spin binary, extracting to {:?}", target_path);
            let mut output_file =
                fs::File::create(target_path).context("Failed to create output file")?;
            std::io::copy(&mut entry, &mut output_file).context("Failed to copy spin binary")?;
            output_file.sync_all()?;
            return Ok(());
        }
    }

    anyhow::bail!("spin binary not found in archive")
}

fn extract_zip(archive_path: &PathBuf, target_path: &PathBuf) -> Result<()> {
    use zip::ZipArchive;

    let file = fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();
        if name.ends_with("spin.exe") || name == "spin.exe" {
            let mut outfile = fs::File::create(target_path)?;
            std::io::copy(&mut file, &mut outfile)?;
            return Ok(());
        }
    }

    anyhow::bail!("spin.exe not found in archive")
}

fn ensure_akamai_plugin(spin_path: &PathBuf) -> Result<()> {
    // Check if Akamai plugin is installed
    let output = Command::new(spin_path)
        .args(["plugin", "list"])
        .output()
        .context("Failed to list Spin plugins")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("aka") {
            debug!("Akamai plugin is already installed");
            return Ok(());
        }
    }

    // Install the plugin
    info!("Installing Akamai plugin for Spin");
    let install_output = Command::new(spin_path)
        .args(["plugin", "install", "aka"])
        .output()
        .context("Failed to install Akamai plugin")?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        eprintln!("⚠️  Warning: Failed to install Akamai plugin: {stderr}");
        eprintln!("   You can install it manually with: spin plugin install aka");
    } else {
        debug!("Akamai plugin installed successfully");
    }

    Ok(())
}

fn get_spin_version(spin_path: &PathBuf) -> Result<String> {
    let output = Command::new(spin_path)
        .arg("--version")
        .output()
        .context("Failed to get spin version")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get spin version");
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    // Parse version from output like "spin 3.3.1 (6fd46d4 2025-06-17)"
    if let Some(version) = version_str.split_whitespace().nth(1) {
        Ok(version.to_string())
    } else {
        anyhow::bail!("Could not parse spin version from: {}", version_str)
    }
}

fn is_version_compatible(actual: &str, required: &str) -> Result<bool> {
    use semver::Version;

    let actual_version = Version::parse(actual)
        .with_context(|| format!("Failed to parse actual version: {actual}"))?;
    let required_version = Version::parse(required)
        .with_context(|| format!("Failed to parse required version: {required}"))?;

    // Check if actual version is >= required version
    Ok(actual_version >= required_version)
}
