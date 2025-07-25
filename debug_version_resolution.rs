use semver::Version;
use std::process::Command;

async fn list_tags_with_crane(repository: &str) -> anyhow::Result<Vec<String>> {
    let output = Command::new("crane")
        .arg("ls")
        .arg(repository)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list tags: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let tags: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    
    Ok(tags)
}

async fn resolve_latest_version(registry_url: &str) -> anyhow::Result<String> {
    println!("Resolving latest version for: {}", registry_url);
    
    // Get all tags from the registry
    let tags = list_tags_with_crane(registry_url).await?;
    println!("Found tags: {:?}", tags);
    
    // Filter and parse semantic versions
    let mut semver_tags: Vec<Version> = Vec::new();
    
    for tag in tags {
        // Skip non-semver tags like "latest", "main", "dev", etc.
        if tag == "latest" || tag == "main" || tag == "master" || tag == "dev" || tag == "edge" {
            println!("Skipping non-semver tag: {}", tag);
            continue;
        }
        
        // Try to parse as semver (with or without 'v' prefix)
        let clean_tag = tag.strip_prefix('v').unwrap_or(&tag);
        match Version::parse(clean_tag) {
            Ok(version) => {
                println!("Parsed semver: {} -> {}", tag, version);
                semver_tags.push(version);
            }
            Err(e) => {
                println!("Failed to parse as semver: {} ({})", tag, e);
            }
        }
    }
    
    if semver_tags.is_empty() {
        anyhow::bail!("No semantic versions found for image at {}", registry_url);
    }
    
    // Sort versions and get the latest
    semver_tags.sort();
    let latest_version = semver_tags.last().unwrap();
    
    println!("Latest version: {}", latest_version);
    Ok(latest_version.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry_url = "ghcr.io/fastertools/ftl-tool-json-formatter";
    match resolve_latest_version(registry_url).await {
        Ok(version) => println!("SUCCESS: Latest version is {}", version),
        Err(e) => println!("ERROR: {}", e)
    }
    Ok(())
}