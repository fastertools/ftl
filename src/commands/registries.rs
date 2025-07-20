use anyhow::{Context, Result};
use clap::Subcommand;
use console::{style, Emoji};

use crate::config::{FtlConfig, registry::{RegistryConfig, RegistryType}};

static CHECK: Emoji<'_, '_> = Emoji("✓", "");
static ARROW: Emoji<'_, '_> = Emoji("→", "->");
static WARN: Emoji<'_, '_> = Emoji("⚠", "!");

#[derive(Subcommand)]
pub enum RegistriesCommand {
    /// List all configured registries
    List,
    
    /// Add a new registry
    Add {
        /// Name for the registry
        name: String,
        
        /// Registry type (ghcr, docker, ecr, custom)
        #[arg(short = 't', long)]
        registry_type: String,
        
        /// Organization (for GHCR)
        #[arg(long)]
        org: Option<String>,
        
        /// AWS Account ID (for ECR)
        #[arg(long)]
        account: Option<String>,
        
        /// AWS Region (for ECR)
        #[arg(long)]
        region: Option<String>,
        
        /// URL pattern (for custom registries)
        #[arg(long)]
        url_pattern: Option<String>,
        
        /// Authentication type (for custom registries: none, basic, bearer)
        #[arg(long)]
        auth_type: Option<String>,
        
        /// Priority for registry searches (lower = higher priority)
        #[arg(long, default_value = "10")]
        priority: u32,
        
        /// Whether to enable the registry immediately
        #[arg(long, default_value = "true")]
        enabled: bool,
    },
    
    /// Remove a registry
    Remove {
        /// Name of the registry to remove
        name: String,
    },
    
    /// Set the default registry
    SetDefault {
        /// Name of the registry to set as default
        name: String,
    },
    
    /// Enable a registry
    Enable {
        /// Name of the registry to enable
        name: String,
    },
    
    /// Disable a registry
    Disable {
        /// Name of the registry to disable
        name: String,
    },
    
    /// Set registry priority
    SetPriority {
        /// Name of the registry
        name: String,
        
        /// New priority (lower = higher priority)
        priority: u32,
    },
}

pub async fn handle_command(cmd: RegistriesCommand) -> Result<()> {
    match cmd {
        RegistriesCommand::List => list_registries().await,
        RegistriesCommand::Add { 
            name, 
            registry_type, 
            org, 
            account, 
            region, 
            url_pattern, 
            auth_type,
            priority,
            enabled,
        } => {
            add_registry(name, registry_type, org, account, region, url_pattern, auth_type, priority, enabled).await
        },
        RegistriesCommand::Remove { name } => remove_registry(name).await,
        RegistriesCommand::SetDefault { name } => set_default_registry(name).await,
        RegistriesCommand::Enable { name } => enable_registry(name, true).await,
        RegistriesCommand::Disable { name } => enable_registry(name, false).await,
        RegistriesCommand::SetPriority { name, priority } => set_priority(name, priority).await,
    }
}

async fn list_registries() -> Result<()> {
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
    println!("{} Use 'ftl registries add' to add a new registry", ARROW);
    
    Ok(())
}

async fn add_registry(
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
    
    // Add to config
    config.add_registry(registry)?;
    config.save()?;
    
    println!("{} Added registry '{}'", CHECK, style(&name).green().bold());
    
    if enabled {
        println!("{} Registry is enabled and ready to use", ARROW);
    } else {
        println!("{} Registry is disabled. Use 'ftl registries enable {}' to enable it", WARN, name);
    }
    
    Ok(())
}

async fn remove_registry(name: String) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.remove_registry(&name)?;
    config.save()?;
    
    println!("{} Removed registry '{}'", CHECK, style(&name).red().bold());
    
    Ok(())
}

async fn set_default_registry(name: String) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.set_default(&name)?;
    config.save()?;
    
    println!("{} Set '{}' as default registry", CHECK, style(&name).green().bold());
    
    Ok(())
}

async fn enable_registry(name: String, enabled: bool) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.set_enabled(&name, enabled)?;
    config.save()?;
    
    let action = if enabled { "Enabled" } else { "Disabled" };
    println!("{} {} registry '{}'", CHECK, action, style(&name).bold());
    
    Ok(())
}

async fn set_priority(name: String, priority: u32) -> Result<()> {
    let mut config = FtlConfig::load()?;
    
    config.set_priority(&name, priority)?;
    config.save()?;
    
    println!("{} Set priority of '{}' to {}", CHECK, style(&name).bold(), priority);
    
    Ok(())
}