use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run `bb write_vk` to generate a verification key from compiled ACIR bytecode.
/// Uses `--oracle_hash keccak` for EVM-compatible verification.
pub fn write_vk(bytecode_path: &Path, output_dir: &Path) -> Result<PathBuf> {
    let output = Command::new("bb")
        .args([
            "write_vk",
            "-b",
            &bytecode_path.display().to_string(),
            "-o",
            &output_dir.display().to_string(),
            "--oracle_hash",
            "keccak",
        ])
        .output()
        .with_context(|| format!(
            "failed to run `bb write_vk` for bytecode {} -- is barretenberg (bb) installed?",
            bytecode_path.display()
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "bb write_vk failed for bytecode {} (output dir: {}):\n{stderr}",
            bytecode_path.display(),
            output_dir.display()
        );
    }

    let vk_path = output_dir.join("vk");
    if !vk_path.exists() {
        bail!(
            "verification key not found at {} after running bb write_vk on {}",
            vk_path.display(),
            bytecode_path.display()
        );
    }

    Ok(vk_path)
}

/// Run `bb write_solidity_verifier` to generate a Solidity verifier contract from a verification key.
pub fn write_solidity_verifier(vk_path: &Path, output_path: &Path) -> Result<()> {
    let output = Command::new("bb")
        .args([
            "write_solidity_verifier",
            "-k",
            &vk_path.display().to_string(),
            "-o",
            &output_path.display().to_string(),
        ])
        .output()
        .with_context(|| format!(
            "failed to run `bb write_solidity_verifier` for vk {} -- is barretenberg (bb) installed?",
            vk_path.display()
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "bb write_solidity_verifier failed for vk {} (output: {}):\n{stderr}",
            vk_path.display(),
            output_path.display()
        );
    }

    if !output_path.exists() {
        bail!(
            "Solidity verifier not found at {} after running bb write_solidity_verifier on {}",
            output_path.display(),
            vk_path.display()
        );
    }

    Ok(())
}
