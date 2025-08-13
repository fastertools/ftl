//! FTL Resolve CLI
//!
//! Component resolver and transpiler for FTL configuration files - resolves registry components, validates syntax, and transpiles to Spin TOML.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ftl_resolve::{FtlConfig, schema_for};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Version information
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[derive(Parser)]
#[command(
    name = "ftl-resolve",
    version = VERSION,
    author = AUTHORS,
    about = DESCRIPTION,
    long_about = format!(
        "{}

This tool resolves and transpiles FTL configuration files - downloads registry components using wkg, \
        validates syntax, generates schemas, and transpiles to Spin TOML format.

\
        Repository: {}

\
        Examples:
  \
        Generate Spin TOML (with wkg resolution):
    \
        ftl-resolve spin ftl.toml -o spin.toml

  \
        Use Spin's native registry resolution:
    \
        ftl-resolve spin ftl.toml -o spin.toml --spin-resolve

  \
        Read from stdin:
    \
        cat ftl.toml | ftl-resolve spin -

  \
        Generate JSON schema:
    \
        ftl-resolve schema -o schema.json

  \
        Validate configuration:
    \
        ftl-resolve validate ftl.toml",
        DESCRIPTION, REPOSITORY
    ),
    after_help = "For more information, see: https://github.com/fastertools/ftl-cli"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Spin TOML from FTL configuration
    #[command(about = "Generate Spin TOML, resolving registry components with wkg by default")]
    Spin {
        /// Input file (ftl.toml or ftl.json, use '-' for stdin)
        input: PathBuf,

        /// Output spin.toml file (writes to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Format of the input (toml or json)
        #[arg(short, long, default_value = "toml")]
        format: InputFormat,

        /// Project directory (for resolving relative paths)
        #[arg(short = 'd', long, default_value = ".")]
        project_dir: PathBuf,

        /// Use Spin's native registry resolution instead of wkg
        #[arg(long)]
        spin_resolve: bool,

        /// Force fresh downloads, ignoring cached components
        #[arg(long)]
        no_cache: bool,
    },
    /// Generate JSON schema for FTL configuration
    #[command(about = "Generate JSON schema for FTL configuration validation")]
    Schema {
        /// Output file for the schema (writes to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output minified JSON instead of pretty-printed
        #[arg(short, long)]
        mini: bool,
    },
    /// Validate an FTL configuration file
    #[command(about = "Validate FTL configuration file syntax and structure")]
    Validate {
        /// Input file to validate (use '-' for stdin)
        input: PathBuf,

        /// Format of the input (toml or json)
        #[arg(short, long, default_value = "toml")]
        format: InputFormat,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum InputFormat {
    Toml,
    Json,
}

fn spin_command(
    input: PathBuf,
    output: Option<PathBuf>,
    format: InputFormat,
    project_dir: PathBuf,
    spin_resolve: bool,
    no_cache: bool,
) -> Result<()> {
    // Read input
    let input_content = if input == Path::new("-") {
        // Explicit stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    } else {
        // File path
        std::fs::read_to_string(&input)
            .with_context(|| format!("Failed to read input file: {}", input.display()))?
    };

    // Parse FTL config based on format
    let ftl_config: FtlConfig = match format {
        InputFormat::Toml => toml::from_str(&input_content).context("Failed to parse TOML")?,
        InputFormat::Json => {
            serde_json::from_str(&input_content).context("Failed to parse JSON")?
        }
    };

    // Validate the configuration
    use garde::Validate;
    ftl_config
        .validate()
        .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

    // Generate Spin TOML
    let spin_toml = if spin_resolve {
        // Use Spin's native registry resolution
        ftl_resolve::transpile_ftl_to_spin(&ftl_config)
            .context("Failed to transpile configuration")?
    } else {
        // Use wkg to resolve registry components
        ftl_resolve::resolve_and_transpile(&ftl_config, &project_dir, no_cache)
            .context("Failed to resolve and transpile configuration")?
    };

    // Write output
    if let Some(path) = output {
        std::fs::write(&path, spin_toml)
            .with_context(|| format!("Failed to write output file: {}", path.display()))?;
        if atty::is(atty::Stream::Stderr) {
            if spin_resolve {
                eprintln!("✓ Successfully wrote spin.toml to {} (using Spin resolution)", path.display());
            } else {
                eprintln!("✓ Successfully resolved components and wrote spin.toml to {}", path.display());
            }
        } else {
            eprintln!("Successfully wrote spin.toml to {}", path.display());
        }
    } else {
        io::stdout()
            .write_all(spin_toml.as_bytes())
            .context("Failed to write to stdout")?;
    }

    Ok(())
}

fn main() -> Result<()> {
    // Enable colored output based on terminal capabilities
    if std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stderr) {
        unsafe {
            std::env::set_var("CLICOLOR", "1");
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Spin {
            input,
            output,
            format,
            project_dir,
            spin_resolve,
            no_cache,
        } => spin_command(input, output, format, project_dir, spin_resolve, no_cache),
        Commands::Schema { output, mini } => generate_schema(output, mini),
        Commands::Validate { input, format } => validate(input, format),
    }
}


fn generate_schema(output: Option<PathBuf>, mini: bool) -> Result<()> {
    let schema = schema_for!(FtlConfig);

    // Default to pretty printing unless mini flag is set
    let json_output = if mini {
        serde_json::to_string(&schema)?
    } else {
        serde_json::to_string_pretty(&schema)?
    };

    if let Some(path) = output {
        std::fs::write(&path, json_output)
            .with_context(|| format!("Failed to write schema file: {}", path.display()))?;
        if atty::is(atty::Stream::Stderr) {
            eprintln!("✓ Successfully wrote schema to {}", path.display());
        } else {
            eprintln!("Successfully wrote schema to {}", path.display());
        }
    } else {
        println!("{}", json_output);
    }

    Ok(())
}

fn validate(input: PathBuf, format: InputFormat) -> Result<()> {
    // Read input
    let input_content = if input == Path::new("-") {
        // Explicit stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    } else {
        // File path
        std::fs::read_to_string(&input)
            .with_context(|| format!("Failed to read input file: {}", input.display()))?
    };

    // Parse FTL config based on format
    let ftl_config: FtlConfig = match format {
        InputFormat::Toml => toml::from_str(&input_content).context("Failed to parse TOML")?,
        InputFormat::Json => {
            serde_json::from_str(&input_content).context("Failed to parse JSON")?
        }
    };

    // Validate the configuration
    use garde::Validate;
    ftl_config
        .validate()
        .map_err(|e| anyhow::anyhow!("Validation failed: {}", e))?;

    if atty::is(atty::Stream::Stderr) {
        eprintln!("✓ Configuration is valid");
    } else {
        eprintln!("Configuration is valid");
    }
    Ok(())
}
