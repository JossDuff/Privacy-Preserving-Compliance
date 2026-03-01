use alloy::primitives::{Address, Bytes};
use alloy::providers::Provider;
use alloy::sol_types::SolValue;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

use crate::eth;
use crate::etherscan;
use crate::etherscan::VerifyArgs;
use crate::forge;
use crate::receipt::Receipt;

#[derive(Debug, Serialize)]
pub struct NewComplianceDefinitionData {
    pub contract_address: String,
    pub transaction_hash: String,
    pub regulator: String,
    pub rpc_url: String,
    pub verification_status: String,
}

pub async fn run(
    rpc_url: &str,
    private_key: &str,
    regulator: &str,
    contract_dir: &Path,
    receipts_dir: &Path,
    verify: &VerifyArgs,
) -> Result<()> {
    let regulator_addr: Address = regulator
        .parse()
        .with_context(|| format!("invalid regulator address: {regulator}"))?;

    eprintln!("compiling contracts...");
    forge::build(contract_dir)?;
    eprintln!("contracts compiled successfully");

    let provider = eth::create_provider(rpc_url, private_key)?;
    let artifact = forge::artifact_path(contract_dir, "ComplianceDefinition.sol", "ComplianceDefinition");

    let constructor_args = Bytes::from(regulator_addr.abi_encode());

    eprintln!("deploying ComplianceDefinition...");
    let result = eth::deploy_from_artifact(&provider, &artifact, Some(constructor_args)).await?;
    eprintln!("deployed to {}", result.deployed_to);

    // Verify via Etherscan API
    let chain_id = provider.get_chain_id().await
        .context("failed to query chain ID from RPC")?;
    let verification = etherscan::verify_contract(
        contract_dir,
        &artifact,
        chain_id,
        &result.deployed_to.to_string(),
        "src/ComplianceDefinition.sol:ComplianceDefinition",
        Some(&alloy::hex::encode(regulator_addr.abi_encode())),
        verify,
    )
    .await?;

    println!("contract_address={}", result.deployed_to);
    println!("transaction_hash={}", result.transaction_hash);
    println!("chain_id={chain_id}");
    println!("verification={verification}");

    let data = NewComplianceDefinitionData {
        contract_address: result.deployed_to.to_string(),
        transaction_hash: result.transaction_hash.to_string(),
        regulator: regulator.to_string(),
        rpc_url: rpc_url.to_string(),
        verification_status: verification.to_string(),
    };

    let receipt = Receipt::new("new-compliance-definition", data);
    receipt.write_to_dir(receipts_dir)?;

    Ok(())
}
