use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub struct ForgeCreateOutput {
    pub deployed_to: String,
    pub transaction_hash: String,
}

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

/// Run `forge create` to deploy a contract and return the deployed address and tx hash.
/// When `verify.etherscan_api_key` is set, the contract is verified on-chain after deployment.
pub fn create(
    project_dir: &Path,
    rpc_url: &str,
    private_key: &str,
    contract: &str,
    constructor_args: &[&str],
    verify: &VerifyArgs,
) -> Result<ForgeCreateOutput> {
    let mut cmd = Command::new("forge");
    cmd.args([
        "create",
        "--root",
        &project_dir.display().to_string(),
        "--rpc-url",
        rpc_url,
        "--private-key",
        private_key,
        contract,
    ]);

    // Verify flags must come before --constructor-args, which is variadic
    // and consumes all remaining positional arguments.
    if let Some(api_key) = verify.etherscan_api_key.as_deref().filter(|k| !k.is_empty()) {
        cmd.args(["--verify", "--etherscan-api-key", api_key]);
        if let Some(url) = verify.verifier_url.as_deref().filter(|u| !u.is_empty()) {
            cmd.args(["--verifier-url", url]);
        }
    }

    if !constructor_args.is_empty() {
        cmd.arg("--constructor-args");
        cmd.args(constructor_args);
    }

    let output = cmd
        .output()
        .with_context(|| format!(
            "failed to run `forge create` for contract {contract} -- is foundry installed?"
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "forge create failed for contract {contract} (rpc: {rpc_url}):\n{stderr}"
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let deployed_to = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Deployed to: "))
        .with_context(|| format!(
            "could not parse deployed address from forge create output for {contract}"
        ))?
        .trim()
        .to_string();

    let transaction_hash = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Transaction hash: "))
        .with_context(|| format!(
            "could not parse transaction hash from forge create output for {contract}"
        ))?
        .trim()
        .to_string();

    Ok(ForgeCreateOutput {
        deployed_to,
        transaction_hash,
    })
}
