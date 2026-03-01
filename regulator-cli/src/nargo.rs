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
    #[serde(rename = "type")]
    package_type: Option<String>,
}

fn read_nargo_toml(project_dir: &Path) -> Result<NargoToml> {
    let toml_path = project_dir.join("Nargo.toml");
    let contents = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("failed to read {}", toml_path.display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("failed to parse {}", toml_path.display()))
}

/// Determine the main source file for a Nargo project based on its package type.
pub fn find_source_file(project_dir: &Path) -> Result<PathBuf> {
    let config = read_nargo_toml(project_dir)?;

    let source_file = match config.package.package_type.as_deref() {
        Some("lib") => project_dir.join("src/lib.nr"),
        _ => project_dir.join("src/main.nr"),
    };

    if !source_file.exists() {
        bail!("source file not found: {}", source_file.display());
    }

    Ok(source_file)
}

/// Run `nargo check` in the given project directory to validate the circuit compiles.
pub fn check(project_dir: &Path) -> Result<()> {
    let output = Command::new("nargo")
        .arg("check")
        .current_dir(project_dir)
        .output()
        .with_context(|| format!(
            "failed to run `nargo check` in {} -- is nargo installed?",
            project_dir.display()
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "nargo check failed in {}:\n{stderr}",
            project_dir.display()
        );
    }

    Ok(())
}

/// Run `nargo compile` in the given project directory and return the path to the compiled JSON.
pub fn compile(project_dir: &Path) -> Result<PathBuf> {
    let output = Command::new("nargo")
        .arg("compile")
        .current_dir(project_dir)
        .output()
        .with_context(|| format!(
            "failed to run `nargo compile` in {} -- is nargo installed?",
            project_dir.display()
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "nargo compile failed in {}:\n{stderr}",
            project_dir.display()
        );
    }

    let config = read_nargo_toml(project_dir)?;

    let bytecode_path = project_dir
        .join("target")
        .join(format!("{}.json", config.package.name));

    if !bytecode_path.exists() {
        bail!(
            "compiled bytecode not found at {} -- did nargo compile succeed for project '{}'?",
            bytecode_path.display(),
            config.package.name
        );
    }

    Ok(bytecode_path)
}
