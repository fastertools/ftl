use anyhow::{Context, Result};
use console::{style, Emoji};

use crate::config::{FtlConfig, registry::{RegistryConfig, RegistryType}};

static CHECK: Emoji<'_, '_> = Emoji("✓", "");
static ARROW: Emoji<'_, '_> = Emoji("→", "->");
static WARN: Emoji<'_, '_> = Emoji("⚠", "!");



pub async fn list_registries() -> Result<()> {
    let config = FtlConfig::load()?;
    
    println!("{} Registry Configuration", style("FTL").bold().cyan());
    println!();
    
    println!("Default registry: {}", style(&config.default_registry).green().bold());
    println!();
    
    println!("{}", style("Configured Registries:").bold());
    
    for registry in &config.registries {
        let status = if registry.enabled {
            style("enabled").green()
        } else {
            style("disabled").dim()
        };
        
        let default_marker = if registry.name == config.default_registry {
            " (default)"
        } else {
            ""
        };
        
        println!(
            "  {} {} [{}] - {} (priority: {}){}",
            if registry.enabled { CHECK } else { WARN },
            style(&registry.name).bold(),
            style(&registry.registry_type.to_string()).cyan(),
            status,
            registry.priority,
            default_marker
        );
        
        // Show display URL if available
        if let Some(url) = &registry.display_url {
            println!("      URL: {}", style(url).blue().underlined());
        }
        
        // Show additional configuration details
        match registry.registry_type {
            RegistryType::Ghcr => {
                if let Some(org) = registry.get_config_str("organization") {
                    println!("      Organization: {}", org);
                }
            }
            RegistryType::Ecr => {
                if let Some(account) = registry.get_config_str("account_id") {
                    println!("      Account ID: {}", account);
                }
                if let Some(region) = registry.get_config_str("region") {
                    println!("      Region: {}", region);
                }
            }
            RegistryType::Custom => {
                if let Some(pattern) = registry.get_config_str("url_pattern") {
                    println!("      URL Pattern: {}", pattern);
                }
                if let Some(auth) = registry.get_config_str("auth_type") {
                    println!("      Auth Type: {}", auth);
                }
            }
            _ => {}
        }
    }
    
    println!();
    println!("{} Use 'ftl registry add' to add a new registry", ARROW);
    
    Ok(())
}

pub async fn add_registry(
    name: String,
    registry_type: String,
    org: Option<String>,
    account: Option<String>,
    region: Option<String>,
    url_pattern: Option<String>,
    auth_type: Option<String>,
    priority: u32,
    enabled: bool,
) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    // Parse registry type
    let reg_type: RegistryType = registry_type.parse()
        .context("Invalid registry type")?;
    
    // Create registry config based on type
    let registry = match reg_type {
        RegistryType::Ghcr => {
            let organization = org.context("--org required for GHCR registries")?;
            RegistryConfig::new_ghcr(name.clone(), organization)
        }
        RegistryType::Docker => {
            RegistryConfig::new_docker(name.clone())
        }
        RegistryType::Ecr => {
            RegistryConfig::new_ecr(name.clone(), account, region)
        }
        RegistryType::Custom => {
            let pattern = url_pattern.context("--url-pattern required for custom registries")?;
            RegistryConfig::new_custom(name.clone(), pattern, auth_type)
        }
    };
    
    // Set priority and enabled status
    let mut registry = registry;
    registry.priority = priority;
    registry.enabled = enabled;
    
    // Clear display_url for user-added registries (will use default from constructors)
    // User can manually edit the config file if they want custom display URLs
    
    // Add to config
    config.add_registry(registry)?;
    config.save()?;
    
    println!("{} Added registry '{}'", CHECK, style(&name).green().bold());
    
    if enabled {
        println!("{} Registry is enabled and ready to use", ARROW);
    } else {
        println!("{} Registry is disabled. Use 'ftl registry enable {}' to enable it", WARN, name);
    }
    
    Ok(())
}

pub async fn remove_registry(name: String) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.remove_registry(&name)?;
    config.save()?;
    
    println!("{} Removed registry '{}'", CHECK, style(&name).red().bold());
    
    Ok(())
}

pub async fn set_default_registry(name: String) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.set_default(&name)?;
    config.save()?;
    
    println!("{} Set '{}' as default registry", CHECK, style(&name).green().bold());
    
    Ok(())
}

pub async fn enable_registry(name: String, enabled: bool) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.set_enabled(&name, enabled)?;
    config.save()?;
    
    let action = if enabled { "Enabled" } else { "Disabled" };
    println!("{} {} registry '{}'", CHECK, action, style(&name).bold());
    
    Ok(())
}

pub async fn set_priority(name: String, priority: u32) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.set_priority(&name, priority)?;
    config.save()?;
    
    println!("{} Set priority of '{}' to {}", CHECK, style(&name).bold(), priority);
    
    Ok(())
}