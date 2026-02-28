use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Deserialize)]
struct NargoToml {
    package: NargoPackage,
}

#[derive(Deserialize)]
struct NargoPackage {
    name: String,
}

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

/// Run `nargo compile` in the given project directory and return the path to the compiled JSON.
pub fn compile(project_dir: &Path) -> Result<PathBuf> {
    let output = Command::new("nargo")
        .arg("compile")
        .current_dir(project_dir)
        .output()
        .context("failed to run nargo compile -- is nargo installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("nargo compile failed:\n{stderr}");
    }

    let nargo_toml = std::fs::read_to_string(project_dir.join("Nargo.toml"))
        .context("failed to read Nargo.toml")?;
    let config: NargoToml = toml::from_str(&nargo_toml).context("failed to parse Nargo.toml")?;

    let bytecode_path = project_dir
        .join("target")
        .join(format!("{}.json", config.package.name));

    if !bytecode_path.exists() {
        bail!(
            "compiled bytecode not found at {}",
            bytecode_path.display()
        );
    }

    Ok(bytecode_path)
}
