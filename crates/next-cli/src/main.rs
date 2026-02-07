mod commands;

use clap::{Parser, Subcommand};
use commands::{create_project, run_build, run_dev_server, run_production_server};

#[derive(Parser)]
#[command(name = "next-rs")]
#[command(about = "Next.js reimplemented in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name } => create_project(&name).await?,
        Commands::Dev { port } => run_dev_server(port).await?,
        Commands::Build => run_build().await?,
        Commands::Start { port } => run_production_server(port).await?,
    }

    Ok(())
}
