use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Run `nargo check` in the given project directory to validate the circuit compiles.
pub fn check(project_dir: &Path) -> Result<()> {
    let output = Command::new("nargo")
        .arg("check")
        .current_dir(project_dir)
        .output()
        .context("failed to run nargo check -- is nargo installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("nargo check failed:\n{stderr}");
    }

    Ok(())
}
