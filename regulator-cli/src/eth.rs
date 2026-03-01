use alloy::{
    hex,
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{Address, Bytes, FixedBytes, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};
use anyhow::{Context, Result};
use std::path::Path;

sol! {
    #[sol(rpc)]
    contract ComplianceDefinition {
        function updateConstraint(
            address newVerifier,
            bytes32 newParamsRoot,
            uint256 tStart,
            uint256 tEnd,
            string calldata metadataHash
        ) external;
    }
}

pub struct DeployOutput {
    pub deployed_to: Address,
    pub transaction_hash: FixedBytes<32>,
}

pub fn create_provider(
    rpc_url: &str,
    private_key: &str,
) -> Result<impl Provider<Ethereum> + Clone> {
    let signer: PrivateKeySigner = private_key
        .parse()
        .context("failed to parse private key")?;

    let url: reqwest::Url = rpc_url
        .parse()
        .with_context(|| format!("invalid RPC URL: {rpc_url}"))?;

    let provider = ProviderBuilder::new()
        .wallet(EthereumWallet::from(signer))
        .connect_http(url);

    Ok(provider)
}

/// Deploy a contract by reading its bytecode from a forge artifact JSON file.
/// If `constructor_args` is provided, it is appended to the bytecode.
pub async fn deploy_from_artifact(
    provider: &(impl Provider<Ethereum> + Clone),
    artifact_path: &Path,
    constructor_args: Option<Bytes>,
) -> Result<DeployOutput> {
    let artifact_bytes = std::fs::read(artifact_path)
        .with_context(|| format!("failed to read artifact: {}", artifact_path.display()))?;

    let artifact: serde_json::Value = serde_json::from_slice(&artifact_bytes)
        .with_context(|| format!("failed to parse artifact JSON: {}", artifact_path.display()))?;

    let bytecode_hex = artifact
        .get("bytecode")
        .and_then(|b| b.get("object"))
        .and_then(|o| o.as_str())
        .with_context(|| {
            format!(
                "missing bytecode.object in artifact: {}",
                artifact_path.display()
            )
        })?;

    let mut bytecode =
        hex::decode(bytecode_hex.strip_prefix("0x").unwrap_or(bytecode_hex)).with_context(
            || {
                format!(
                    "invalid hex in bytecode.object of artifact: {}",
                    artifact_path.display()
                )
            },
        )?;

    if let Some(args) = constructor_args {
        bytecode.extend_from_slice(&args);
    }

    let tx = <Ethereum as alloy::network::Network>::TransactionRequest::default()
        .with_deploy_code(Bytes::from(bytecode));

    let pending_tx = provider
        .send_transaction(tx)
        .await
        .context("failed to broadcast contract deployment")?;

    let tx_hash = *pending_tx.tx_hash();

    let receipt = pending_tx
        .get_receipt()
        .await
        .context("contract deployment transaction failed")?;

    let deployed_to = receipt
        .contract_address
        .context("no contract address in deployment receipt")?;

    Ok(DeployOutput {
        deployed_to,
        transaction_hash: tx_hash,
    })
}

pub async fn call_update_constraint(
    provider: &(impl Provider<Ethereum> + Clone),
    compliance_definition_addr: Address,
    new_verifier: Address,
    params_root: FixedBytes<32>,
    t_start: U256,
    t_end: U256,
    metadata_uri: String,
) -> Result<FixedBytes<32>> {
    let contract = ComplianceDefinition::new(compliance_definition_addr, provider);

    let pending_tx = contract
        .updateConstraint(new_verifier, params_root, t_start, t_end, metadata_uri)
        .send()
        .await
        .context("failed to broadcast updateConstraint transaction")?;

    let tx_hash = *pending_tx.tx_hash();

    pending_tx
        .get_receipt()
        .await
        .context("updateConstraint transaction failed")?;

    Ok(tx_hash)
}
