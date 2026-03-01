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
        .context("failed to run forge build -- is foundry installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge build failed:\n{stderr}");
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

    if !constructor_args.is_empty() {
        cmd.arg("--constructor-args");
        cmd.args(constructor_args);
    }

    if let Some(api_key) = &verify.etherscan_api_key {
        cmd.args(["--verify", "--etherscan-api-key", api_key]);
        if let Some(url) = &verify.verifier_url {
            cmd.args(["--verifier-url", url]);
        }
    }

    let output = cmd
        .output()
        .context("failed to run forge create -- is foundry installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge create failed:\n{stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let deployed_to = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Deployed to: "))
        .context("could not parse deployed address from forge create output")?
        .trim()
        .to_string();

    let transaction_hash = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Transaction hash: "))
        .context("could not parse transaction hash from forge create output")?
        .trim()
        .to_string();

    Ok(ForgeCreateOutput {
        deployed_to,
        transaction_hash,
    })
}
