use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod bb;
mod cast;
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

const UINT256_MAX: &str =
    "115792089237316195423570985008687907853269984665640564039457584007913129639935";
const BYTES32_ZERO: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

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
    /// Validate, compile, deploy a Noir circuit verifier, and register it with a ComplianceDefinition
    Publish {
        /// Path to the Noir project directory (containing Nargo.toml)
        #[arg(value_name = "DIR")]
        path: PathBuf,

        /// RPC URL of the target chain
        #[arg(long, env = "RPC_URL")]
        rpc_url: String,

        /// Private key for the deployer account
        #[arg(long, env = "PRIVATE_KEY")]
        private_key: String,

        /// Address of the deployed ComplianceDefinition contract
        #[arg(long)]
        compliance_definition: String,

        /// Path to write the generated Solidity verifier [default: <DIR>/target/Verifier.sol]
        #[arg(long, value_name = "FILE")]
        verifier_output: Option<PathBuf>,

        /// Path to the Foundry project for deploying the verifier
        #[arg(long, default_value = "verifier-base-contract", value_name = "DIR")]
        contract_dir: PathBuf,

        /// Merkle root of public parameters (bytes32)
        #[arg(long, default_value = BYTES32_ZERO)]
        params_root: String,

        /// Block height when this version becomes active
        #[arg(long, default_value = "0")]
        t_start: String,

        /// Block height when this version expires
        #[arg(long, default_value = UINT256_MAX)]
        t_end: String,
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
            rpc_url,
            private_key,
            compliance_definition,
            verifier_output,
            contract_dir,
            params_root,
            t_start,
            t_end,
        } => {
            commands::publish::run(
                path,
                verifier_output,
                &ipfs_url,
                &rpc_url,
                &private_key,
                &compliance_definition,
                &contract_dir,
                &params_root,
                &t_start,
                &t_end,
                &output,
            )
            .await
        }
        Commands::Update => commands::update::run().await,
    }
}
