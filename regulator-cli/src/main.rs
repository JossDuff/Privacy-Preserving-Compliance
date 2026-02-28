use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod ipfs;
mod nargo;
mod receipt;

#[derive(Parser)]
#[command(name = "regulator-cli")]
#[command(about = "CLI for managing privacy-preserving compliance definitions")]
struct Cli {
    /// IPFS RPC endpoint URL
    #[arg(long, global = true, env = "IPFS_RPC_URL")]
    ipfs_rpc_url: Option<String>,

    /// Path to write the JSON receipt summarizing the operation [default: ./receipt.json]
    #[arg(short, long, global = true, value_name = "FILE")]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate and upload a Noir circuit to IPFS as a new compliance definition
    NewComplianceDefinition {
        /// Path to the Noir project directory (containing Nargo.toml)
        #[arg(value_name = "DIR")]
        path: PathBuf,
    },
    /// Initialize a new Noir compliance definition project
    Init {
        /// Name for the new project
        name: String,
    },
    /// Publish a compliance definition (deploy verifier contract) TODO
    Publish,
    /// Update an existing compliance definition TODO
    Update,
}

const DEFAULT_IPFS_RPC_URL: &str = "http://localhost:5001";
const DEFAULT_RECEIPT_PATH: &str = "receipt.json";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let ipfs_url = cli
        .ipfs_rpc_url
        .unwrap_or_else(|| DEFAULT_IPFS_RPC_URL.to_string());

    let output = cli
        .output
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RECEIPT_PATH));

    match cli.command {
        Commands::NewComplianceDefinition { path } => {
            commands::new_compliance_definition::run(path, &ipfs_url, &output).await
        }
        Commands::Init { name } => commands::init::run(&name).await,
        Commands::Publish => commands::publish::run().await,
        Commands::Update => commands::update::run().await,
    }
}
