use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Optional Etherscan/block-explorer verification settings.
#[derive(Clone, Default)]
pub struct VerifyArgs {
    pub etherscan_api_key: Option<String>,
    pub verifier_url: Option<String>,
}

/// Run `forge build` to compile the Solidity contracts in the given project directory.
pub fn build(project_dir: &Path) -> Result<()> {
    let output = Command::new("forge")
        .args(["build", "--root", &project_dir.display().to_string()])
        .output()
        .with_context(|| format!(
            "failed to run `forge build` for {} -- is foundry installed?",
            project_dir.display()
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "forge build failed for project {}:\n{stderr}",
            project_dir.display()
        );
    }

    Ok(())
}

/// Return the path to a forge build artifact JSON for a given contract.
pub fn artifact_path(project_dir: &Path, sol_file: &str, contract_name: &str) -> PathBuf {
    project_dir
        .join("out")
        .join(sol_file)
        .join(format!("{contract_name}.json"))
}

/// Run `forge verify-contract` to verify a deployed contract on a block explorer.
pub fn verify_contract(
    project_dir: &Path,
    rpc_url: &str,
    contract_address: &str,
    contract: &str,
    constructor_args: Option<&str>,
    verify: &VerifyArgs,
) -> Result<()> {
    let api_key = match verify.etherscan_api_key.as_deref().filter(|k| !k.is_empty()) {
        Some(key) => key,
        None => return Ok(()),
    };

    let mut cmd = Command::new("forge");
    cmd.args([
        "verify-contract",
        "--root",
        &project_dir.display().to_string(),
        "--rpc-url",
        rpc_url,
        "--etherscan-api-key",
        api_key,
        contract_address,
        contract,
    ]);

    if let Some(args) = constructor_args {
        cmd.args(["--constructor-args", args]);
    }

    if let Some(url) = verify.verifier_url.as_deref().filter(|u| !u.is_empty()) {
        cmd.args(["--verifier-url", url]);
    }

    // Prevent forge from picking up empty env vars.
    if verify.verifier_url.as_deref().is_none_or(|u| u.is_empty()) {
        cmd.env_remove("VERIFIER_URL");
    }

    let output = cmd
        .output()
        .with_context(|| format!(
            "failed to run `forge verify-contract` for {contract} -- is foundry installed?"
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("warning: contract verification failed for {contract}:\n{stderr}");
    }

    Ok(())
}
