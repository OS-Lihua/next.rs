mod commands;
mod config;

use clap::{Parser, Subcommand};
use commands::{
    add_component, add_layout, add_page, create_project, run_build, run_check, run_dev_server,
    run_production_server,
};

#[derive(Parser)]
#[command(name = "next")]
#[command(about = "Next.js reimplemented in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, clap::ValueEnum)]
enum AddType {
    Page,
    Layout,
    Component,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new next.rs project
    Create {
        /// Project name
        name: String,
    },
    /// Start development server
    Dev {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Build for production
    Build,
    /// Start production server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Add a new page, layout, or component
    Add {
        /// Type of item to add
        #[arg(value_enum)]
        item_type: AddType,
        /// Path or name (e.g., /dashboard, sidebar)
        name: String,
    },
    /// Check project for errors
    Check {
        /// Output errors in JSON format (for AI agents)
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name } => create_project(&name).await?,
        Commands::Dev { port } => run_dev_server(port).await?,
        Commands::Build => run_build().await?,
        Commands::Start { port } => run_production_server(port).await?,
        Commands::Add { item_type, name } => match item_type {
            AddType::Page => add_page(&name).await?,
            AddType::Layout => add_layout(&name).await?,
            AddType::Component => add_component(&name).await?,
        },
        Commands::Check { json } => run_check(json).await?,
    }

    Ok(())
}
