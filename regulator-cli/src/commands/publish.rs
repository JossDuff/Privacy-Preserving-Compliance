use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::bb;
use crate::cast;
use crate::forge;
use crate::ipfs;
use crate::nargo;
use crate::receipt::Receipt;

#[derive(Debug, Serialize)]
pub struct PublishData {
    pub project_dir: String,
    pub bytecode_path: String,
    pub vk_path: String,
    pub verifier_path: String,
    pub cid: String,
    pub file_name: String,
    pub ipfs_size: String,
    pub verifier_address: String,
    pub deploy_tx_hash: String,
    pub compliance_definition: String,
    pub update_tx_hash: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    project_dir: PathBuf,
    verifier_output: Option<PathBuf>,
    ipfs_rpc_url: &str,
    rpc_url: &str,
    private_key: &str,
    compliance_definition: &str,
    contract_dir: &Path,
    params_root: &str,
    t_start: &str,
    t_end: &str,
    receipts_dir: &Path,
) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("not a directory: {}", project_dir.display());
    }

    if !project_dir.join("Nargo.toml").exists() {
        bail!(
            "no Nargo.toml found in {} -- is this a Noir project?",
            project_dir.display()
        );
    }

    let source_file = nargo::find_source_file(&project_dir)?;

    // 1. Validate circuit
    eprintln!("validating circuit...");
    nargo::check(&project_dir).context("circuit validation failed")?;
    eprintln!("circuit validated successfully");

    // 2. Compile the circuit
    eprintln!("compiling circuit...");
    let bytecode_path = nargo::compile(&project_dir)?;
    eprintln!("circuit compiled successfully");

    // 3. Generate verification key
    let target_dir = project_dir.join("target");
    eprintln!("generating verification key...");
    let vk_path = bb::write_vk(&bytecode_path, &target_dir)?;
    eprintln!("verification key generated");

    // 4. Generate Solidity verifier
    let verifier_path = verifier_output.unwrap_or_else(|| target_dir.join("Verifier.sol"));
    eprintln!("generating Solidity verifier...");
    bb::write_solidity_verifier(&vk_path, &verifier_path)?;
    eprintln!("Solidity verifier generated");

    // 5. Upload circuit source to IPFS
    eprintln!("uploading circuit to IPFS...");
    let response = ipfs::add_file(ipfs_rpc_url, &source_file)
        .await
        .context("failed to upload circuit to IPFS")?;
    eprintln!("uploaded to IPFS");

    // 6. Copy Verifier.sol into the Foundry project for deployment
    let deploy_verifier_path = contract_dir.join("src/Verifier.sol");
    std::fs::copy(&verifier_path, &deploy_verifier_path).with_context(|| {
        format!(
            "failed to copy Verifier.sol to {}",
            deploy_verifier_path.display()
        )
    })?;

    // 7. Build the Foundry project with the new Verifier.sol
    eprintln!("compiling verifier contract...");
    forge::build(contract_dir)?;
    eprintln!("verifier contract compiled");

    // 8. Deploy the HonkVerifier contract
    eprintln!("deploying verifier contract...");
    let deploy_result = forge::create(
        contract_dir,
        rpc_url,
        private_key,
        "src/Verifier.sol:HonkVerifier",
        &[],
    )?;
    eprintln!("verifier deployed to {}", deploy_result.deployed_to);

    // 9. Call updateConstraint on the ComplianceDefinition contract
    let cid = &response.hash;
    eprintln!("registering compliance version...");
    let update_result = cast::send(
        rpc_url,
        private_key,
        compliance_definition,
        "updateConstraint(address,bytes32,uint256,uint256,string)",
        &[
            &deploy_result.deployed_to,
            params_root,
            t_start,
            t_end,
            cid,
        ],
    )?;
    eprintln!("compliance version registered");

    println!("{}", deploy_result.deployed_to);

    let data = PublishData {
        project_dir: project_dir.display().to_string(),
        bytecode_path: bytecode_path.display().to_string(),
        vk_path: vk_path.display().to_string(),
        verifier_path: verifier_path.display().to_string(),
        cid: cid.to_string(),
        file_name: response.name,
        ipfs_size: response.size,
        verifier_address: deploy_result.deployed_to,
        deploy_tx_hash: deploy_result.transaction_hash,
        compliance_definition: compliance_definition.to_string(),
        update_tx_hash: update_result.transaction_hash,
    };

    let receipt = Receipt::new("publish", data);
    receipt.write_to_dir(receipts_dir)?;

    Ok(())
}
