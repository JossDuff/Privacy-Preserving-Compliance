use alloy::primitives::{Address, FixedBytes};
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::eth;
use crate::ipfs;
use crate::receipt::Receipt;

#[derive(Debug, Serialize)]
pub struct UpdateParamsData {
    pub compliance_definition: String,
    pub merkle_root: String,
    pub leaves_file: String,
    pub leaves_cid: String,
    pub update_tx_hash: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    compliance_definition: &str,
    ipfs_rpc_url: &str,
    rpc_url: &str,
    private_key: &str,
    merkle_root: &str,
    leaves_file: Option<PathBuf>,
    leaves_cid_override: Option<String>,
    receipts_dir: &Path,
) -> Result<()> {
    // 1. Resolve the leaves CID: either upload, or use the pre-pinned override.
    let (leaves_cid, leaves_file_display) = if let Some(cid) = leaves_cid_override {
        eprintln!("using pre-pinned leaves CID: {cid}");
        (cid, String::new())
    } else {
        let leaves_path = leaves_file
            .as_ref()
            .context("either --leaves-file or --leaves-cid must be provided")?;
        eprintln!("uploading leaves file {}...", leaves_path.display());
        let leaves_response = ipfs::add_file(ipfs_rpc_url, leaves_path)
            .await
            .with_context(|| {
                format!("failed to upload leaves file to IPFS at {ipfs_rpc_url}")
            })?;
        eprintln!("leaves uploaded to IPFS: {}", leaves_response.hash);
        (leaves_response.hash, leaves_path.display().to_string())
    };

    // 2. Call updateParams on the ComplianceDefinition contract
    let cd_addr: Address = compliance_definition
        .parse()
        .with_context(|| format!("invalid compliance definition address: {compliance_definition}"))?;
    let merkle_root_bytes: FixedBytes<32> = merkle_root
        .parse()
        .with_context(|| format!("invalid merkle_root (expected bytes32): {merkle_root}"))?;

    let provider = eth::create_provider(rpc_url, private_key)?;

    eprintln!("calling updateParams...");
    let update_tx_hash = eth::call_update_params(
        &provider,
        cd_addr,
        merkle_root_bytes,
        leaves_cid.to_string(),
    )
    .await?;
    eprintln!("updateParams succeeded");

    println!("compliance_definition={compliance_definition}");
    println!("merkle_root={merkle_root}");
    println!("leaves_cid={leaves_cid}");
    println!("update_tx_hash={update_tx_hash}");

    let data = UpdateParamsData {
        compliance_definition: compliance_definition.to_string(),
        merkle_root: merkle_root.to_string(),
        leaves_file: leaves_file_display,
        leaves_cid: leaves_cid.to_string(),
        update_tx_hash: update_tx_hash.to_string(),
    };

    let receipt = Receipt::new("update-params", data);
    receipt.write_to_dir(receipts_dir)?;

    Ok(())
}
