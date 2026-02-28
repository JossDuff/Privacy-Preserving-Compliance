use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod ipfs;

#[derive(Parser)]
#[command(name = "regulator-cli")]
#[command(about = "CLI for managing privacy-preserving compliance definitions")]
struct Cli {
    /// IPFS RPC endpoint URL
    #[arg(long, global = true, env = "IPFS_RPC_URL")]
    ipfs_rpc_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a Noir circuit to IPFS as a new compliance definition
    NewComplianceDefinition {
        /// Path to the .nr circuit file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Initialize a new compliance definition project TODO
    Init,
    /// Publish a compliance definition (deploy verifier contract) TODO
    Publish,
    /// Update an existing compliance definition TODO
    Update,
}

const DEFAULT_IPFS_RPC_URL: &str = "http://localhost:5001";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let ipfs_url = cli
        .ipfs_rpc_url
        .unwrap_or_else(|| DEFAULT_IPFS_RPC_URL.to_string());

    match cli.command {
        Commands::NewComplianceDefinition { file } => {
            commands::new_compliance_definition::run(file, &ipfs_url).await
        }
        Commands::Init => commands::init::run().await,
        Commands::Publish => commands::publish::run().await,
        Commands::Update => commands::update::run().await,
    }
}
