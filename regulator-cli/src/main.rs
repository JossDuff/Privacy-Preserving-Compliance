use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod bb;
mod commands;
mod forge;
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
    /// Deploy a new ComplianceDefinition contract on-chain
    NewComplianceDefinition {
        /// RPC URL of the target chain
        #[arg(long, env = "RPC_URL")]
        rpc_url: String,

        /// Private key for the deployer account
        #[arg(long, env = "PRIVATE_KEY")]
        private_key: String,

        /// Address of the regulator that will control the compliance definition
        #[arg(long)]
        regulator: String,

        /// Path to the Foundry project containing ComplianceDefinition.sol
        #[arg(long, default_value = "verifier-base-contract", value_name = "DIR")]
        contract_dir: PathBuf,
    },
    /// Initialize a new Noir compliance definition project
    Init {
        /// Name for the new project
        name: String,
    },
    /// Validate, compile, and publish a Noir circuit as a Solidity verifier
    Publish {
        /// Path to the Noir project directory (containing Nargo.toml)
        #[arg(value_name = "DIR")]
        path: PathBuf,

        /// Path to write the generated Solidity verifier [default: <DIR>/target/Verifier.sol]
        #[arg(long, value_name = "FILE")]
        verifier_output: Option<PathBuf>,
    },
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
        Commands::NewComplianceDefinition {
            rpc_url,
            private_key,
            regulator,
            contract_dir,
        } => {
            commands::new_compliance_definition::run(
                &rpc_url,
                &private_key,
                &regulator,
                &contract_dir,
                &output,
            )
            .await
        }
        Commands::Init { name } => commands::init::run(&name).await,
        Commands::Publish {
            path,
            verifier_output,
        } => commands::publish::run(path, verifier_output, &ipfs_url, &output).await,
        Commands::Update => commands::update::run().await,
    }
}
