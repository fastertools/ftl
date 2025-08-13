use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use which::which;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "spin-compose",
    bin_name = "spinc",
    version = VERSION,
    about = "Infrastructure as Code for WebAssembly - compose and synthesize Spin applications",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new spin-compose project
    Init {
        /// Project name
        #[arg(default_value = "my-app")]
        name: String,
        
        /// Template to use
        #[arg(short, long, default_value = "mcp")]
        template: String,
    },
    
    /// Synthesize spin.toml from configuration
    Synth {
        /// Input configuration file
        #[arg(default_value = "spinc.yaml")]
        input: PathBuf,
        
        /// Output file (defaults to spin.toml)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Environment/stack to use
        #[arg(short, long)]
        env: Option<String>,
        
        /// Set configuration values (can be used multiple times)
        #[arg(short, long)]
        set: Vec<String>,
    },
    
    /// Show what would change in spin.toml
    Diff {
        /// Input configuration file
        #[arg(default_value = "spinc.yaml")]
        input: PathBuf,
        
        /// Existing spin.toml to compare against
        #[arg(default_value = "spin.toml")]
        current: PathBuf,
    },
    
    /// Validate configuration
    Validate {
        /// Input configuration file
        #[arg(default_value = "spinc.yaml")]
        input: PathBuf,
    },
    
    /// Manage constructs
    Construct {
        #[command(subcommand)]
        action: ConstructAction,
    },
}

#[derive(Subcommand)]
enum ConstructAction {
    /// List available constructs
    List,
    
    /// Add a construct to your project
    Add {
        /// Construct name (e.g., fermyon/mcp)
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check if CUE is installed
    check_cue_installed()?;
    
    match cli.command {
        Commands::Init { name, template } => init_project(&name, &template),
        Commands::Synth { input, output, env, set } => {
            synthesize(&input, output.as_deref(), env.as_deref(), &set)
        },
        Commands::Diff { input, current } => diff(&input, &current),
        Commands::Validate { input } => validate(&input),
        Commands::Construct { action } => match action {
            ConstructAction::List => list_constructs(),
            ConstructAction::Add { name } => add_construct(&name),
        },
    }
}

fn check_cue_installed() -> Result<()> {
    if which("cue").is_err() {
        eprintln!("{}: CUE is not installed", "Error".red().bold());
        eprintln!();
        eprintln!("spin-compose requires CUE to be installed.");
        eprintln!("Install it using one of these methods:");
        eprintln!();
        eprintln!("  {} brew install cue", "macOS:".cyan());
        eprintln!("  {} go install cuelang.org/go/cmd/cue@latest", "Go:".cyan());
        eprintln!("  {} Download from https://github.com/cue-lang/cue/releases", "Binary:".cyan());
        eprintln!();
        eprintln!("For more information: https://cuelang.org/docs/install/");
        std::process::exit(1);
    }
    Ok(())
}

fn init_project(name: &str, template: &str) -> Result<()> {
    println!("{} {} project '{}'", 
        "Initializing".green().bold(),
        template,
        name
    );
    
    // Create spinc.yaml based on template
    let config = match template {
        "mcp" => format!(
            r#"# spin-compose configuration
name: {}
version: 0.1.0
description: MCP application

# Template
template: mcp

# Authentication (set enabled: false to disable)
auth:
  enabled: true
  issuer: https://auth.example.com
  audience:
    - api.example.com

# MCP configuration
mcp:
  gateway: ghcr.io/fastertools/mcp-gateway:latest
  authorizer: ghcr.io/fastertools/mcp-authorizer:latest
  validate_arguments: false

# Components
components:
  # example-tool:
  #   source: ./build/example.wasm
  #   route: /example
  #   build:
  #     command: cargo build --target wasm32-wasip1 --release
  #     watch:
  #       - src/**/*.rs
  #       - Cargo.toml
"#,
            name
        ),
        _ => {
            eprintln!("{}: Unknown template '{}'", "Error".red().bold(), template);
            std::process::exit(1);
        }
    };
    
    // Write config file
    fs::write("spinc.yaml", config)?;
    println!("{} spinc.yaml", "Created".green());
    
    // Create .gitignore if it doesn't exist
    if !Path::new(".gitignore").exists() {
        fs::write(".gitignore", "spin.toml\n.spin/\n")?;
        println!("{} .gitignore", "Created".green());
    }
    
    println!();
    println!("{}", "Next steps:".cyan().bold());
    println!("  1. Edit spinc.yaml to configure your application");
    println!("  2. Run 'spinc synth' to generate spin.toml");
    println!("  3. Run 'spin up' to start your application");
    
    Ok(())
}

fn synthesize(
    input: &Path,
    output: Option<&Path>,
    _env: Option<&str>,
    _set: &[String],
) -> Result<()> {
    println!("{} {}", 
        "Synthesizing".green().bold(),
        input.display()
    );
    
    // Read input file
    let _input_content = fs::read_to_string(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;
    
    // Determine input format by extension
    let _input_format = match input.extension().and_then(|s| s.to_str()) {
        Some("yaml") | Some("yml") => "yaml",
        Some("json") => "json",
        Some("toml") => "toml",
        Some("cue") => "cue",
        _ => "yaml", // Default
    };
    
    // Create a temporary CUE file that imports and transforms the input
    let transform_cue = format!(
        r#"
import "spinc.io/solutions/mcp"

// Import the input configuration
input: {{}}

// Create MCP application
app: mcp.#McpApplication & input

// Export the synthesized manifest
output: app.manifest
"#
    );
    
    // Write transform file
    let transform_file = NamedTempFile::new()?;
    fs::write(transform_file.path(), transform_cue)?;
    
    // Get the spin-compose directory for CUE modules
    let exe_path = std::env::current_exe()?;
    let spinc_dir = exe_path
        .parent()  // bin directory
        .and_then(|p| p.parent())  // target directory
        .and_then(|p| p.parent())  // project root
        .map(|p| p.join("crates/spin-compose"))
        .ok_or_else(|| anyhow::anyhow!("Could not find spin-compose directory"))?;
    
    // Get absolute path for input file
    let input_abs = input.canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", input.display()))?;
    
    // Run CUE to synthesize
    let cue_output = Command::new("cue")
        .current_dir(&spinc_dir)
        .args(&[
            "export",
            transform_file.path().to_str().unwrap(),
            input_abs.to_str().unwrap(),
            "-e", "output",
            "--out", "toml",
            "-p", "main",
        ])
        .env("CUE_EXPERIMENT", "modules")
        .output()
        .context("Failed to run CUE")?;
    
    if !cue_output.status.success() {
        let stderr = String::from_utf8_lossy(&cue_output.stderr);
        eprintln!("{}: CUE synthesis failed", "Error".red().bold());
        eprintln!("{}", stderr);
        std::process::exit(1);
    }
    
    let spin_toml = String::from_utf8(cue_output.stdout)?;
    
    // Write output
    let output_path = output.unwrap_or(Path::new("spin.toml"));
    fs::write(output_path, spin_toml)?;
    
    println!("{} {}", 
        "Generated".green().bold(),
        output_path.display()
    );
    
    Ok(())
}

fn diff(input: &Path, current: &Path) -> Result<()> {
    // First synthesize to a temp file
    let temp_file = NamedTempFile::new()?;
    synthesize(input, Some(temp_file.path()), None, &[])?;
    
    // Read both files
    let current_content = fs::read_to_string(current)
        .unwrap_or_else(|_| String::new());
    let new_content = fs::read_to_string(temp_file.path())?;
    
    if current_content == new_content {
        println!("{}", "No changes detected".green());
    } else {
        println!("{}", "Changes detected:".yellow().bold());
        println!();
        
        // Simple line-by-line diff
        let current_lines: Vec<&str> = current_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        for (i, (old, new)) in current_lines.iter()
            .zip(new_lines.iter())
            .enumerate()
        {
            if old != new {
                println!("{}: {}", 
                    format!("Line {}", i + 1).cyan(),
                    format!("- {}", old).red()
                );
                println!("{}: {}", 
                    format!("     {}", " ".repeat(i.to_string().len())).cyan(),
                    format!("+ {}", new).green()
                );
            }
        }
        
        // Handle different lengths
        if current_lines.len() < new_lines.len() {
            for line in &new_lines[current_lines.len()..] {
                println!("{}", format!("+ {}", line).green());
            }
        } else if current_lines.len() > new_lines.len() {
            for line in &current_lines[new_lines.len()..] {
                println!("{}", format!("- {}", line).red());
            }
        }
    }
    
    Ok(())
}

fn validate(input: &Path) -> Result<()> {
    println!("{} {}", 
        "Validating".cyan().bold(),
        input.display()
    );
    
    // Create validation CUE
    let validate_cue = r#"
import "spinc.io/solutions/mcp"

// Import and validate against schema
input: mcp.#McpApplication
"#;
    
    // Write validation file
    let validate_file = NamedTempFile::new()?;
    fs::write(validate_file.path(), validate_cue)?;
    
    // Get the spin-compose directory for CUE modules
    let exe_path = std::env::current_exe()?;
    let spinc_dir = exe_path
        .parent()  // bin directory
        .and_then(|p| p.parent())  // target directory
        .and_then(|p| p.parent())  // project root
        .map(|p| p.join("crates/spin-compose"))
        .ok_or_else(|| anyhow::anyhow!("Could not find spin-compose directory"))?;
    
    // Get absolute path for input file
    let input_abs = input.canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", input.display()))?;
    
    // Run CUE validation
    let output = Command::new("cue")
        .current_dir(&spinc_dir)
        .args(&[
            "vet",
            validate_file.path().to_str().unwrap(),
            input_abs.to_str().unwrap(),
            "-p", "main",
        ])
        .env("CUE_EXPERIMENT", "modules")
        .output()
        .context("Failed to run CUE")?;
    
    if output.status.success() {
        println!("{} Configuration is valid", "✓".green().bold());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{} Validation failed:", "✗".red().bold());
        eprintln!("{}", stderr);
        std::process::exit(1);
    }
    
    Ok(())
}

fn list_constructs() -> Result<()> {
    println!("{}", "Available constructs:".cyan().bold());
    println!();
    
    println!("  {} - MCP application (authentication, gateway, components)", "mcp".green());
    println!("  {} - WordPress site", "wordpress".dimmed());
    println!("  {} - Microservices mesh", "microservices".dimmed());
    println!("  {} - AI pipeline", "ai-pipeline".dimmed());
    
    println!();
    println!("{}", "More constructs coming soon!".dimmed());
    
    Ok(())
}

fn add_construct(name: &str) -> Result<()> {
    println!("{} construct '{}'", 
        "Adding".green().bold(),
        name
    );
    
    // This would download/install constructs from a registry
    // For now, just show a message
    println!("{}: Construct registry not yet implemented", "Info".yellow());
    println!("Available constructs are bundled with spin-compose");
    
    Ok(())
}