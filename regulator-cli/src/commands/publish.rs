use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::bb;
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
}

pub async fn run(
    project_dir: PathBuf,
    verifier_output: Option<PathBuf>,
    ipfs_rpc_url: &str,
    output: &Path,
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

    println!("{}", response.hash);

    let data = PublishData {
        project_dir: project_dir.display().to_string(),
        bytecode_path: bytecode_path.display().to_string(),
        vk_path: vk_path.display().to_string(),
        verifier_path: verifier_path.display().to_string(),
        cid: response.hash,
        file_name: response.name,
        ipfs_size: response.size,
    };

    let receipt = Receipt::new("publish", data);
    receipt.write_to(output)?;

    Ok(())
}
