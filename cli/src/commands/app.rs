use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: AppCommand,
}

#[derive(Debug, Subcommand)]
pub enum AppCommand {
    /// List all applications
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Get status of an application
    Status {
        /// Application name
        app_name: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Delete an application
    Delete {
        /// Application name
        app_name: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

pub async fn execute(args: AppArgs) -> Result<()> {
    let command = match args.command {
        AppCommand::List { format } => ftl_commands::app::AppCommand::List {
            format: format.into(),
        },
        AppCommand::Status { app_name, format } => ftl_commands::app::AppCommand::Status {
            app_name,
            format: format.into(),
        },
        AppCommand::Delete { app_name, force } => {
            ftl_commands::app::AppCommand::Delete { app_name, force }
        }
    };

    let cmd_args = ftl_commands::app::AppArgs { command };

    ftl_commands::app::execute(cmd_args).await
}

impl From<OutputFormat> for ftl_commands::app::OutputFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Table => ftl_commands::app::OutputFormat::Table,
            OutputFormat::Json => ftl_commands::app::OutputFormat::Json,
        }
    }
}
