use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run `forge build` to compile the Solidity contracts in the given project directory.
pub fn build(project_dir: &Path) -> Result<()> {
    let output = Command::new("forge")
        .args([
            "build",
            "--root",
            &project_dir.display().to_string(),
            "--optimize",
            "--optimizer-runs",
            "1",
        ])
        .output()
        .with_context(|| {
            format!(
                "failed to run `forge build` for {} -- is foundry installed?",
                project_dir.display()
            )
        })?;

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
