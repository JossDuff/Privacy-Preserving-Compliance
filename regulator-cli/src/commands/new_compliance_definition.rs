use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

use crate::ipfs;
use crate::nargo;
use crate::receipt::Receipt;

#[derive(Debug, Serialize)]
pub struct NewComplianceDefinitionData {
    pub cid: String,
    pub file_name: String,
    pub file_path: String,
    pub project_dir: String,
    pub ipfs_size: String,
}

#[derive(Deserialize)]
struct NargoToml {
    package: NargoPackage,
}

#[derive(Deserialize)]
struct NargoPackage {
    #[serde(rename = "type")]
    package_type: Option<String>,
}

/// Determine the main source file for a Nargo project based on its package type.
fn find_source_file(project_dir: &PathBuf) -> Result<PathBuf> {
    let nargo_toml_path = project_dir.join("Nargo.toml");
    let contents = std::fs::read_to_string(&nargo_toml_path)
        .with_context(|| format!("failed to read {}", nargo_toml_path.display()))?;

    let config: NargoToml =
        toml::from_str(&contents).context("failed to parse Nargo.toml")?;

    let source_file = match config.package.package_type.as_deref() {
        Some("lib") => project_dir.join("src/lib.nr"),
        _ => project_dir.join("src/main.nr"),
    };

    if !source_file.exists() {
        bail!("source file not found: {}", source_file.display());
    }

    Ok(source_file)
}

pub async fn run(project_dir: PathBuf, ipfs_rpc_url: &str, output: &PathBuf) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("not a directory: {}", project_dir.display());
    }

    if !project_dir.join("Nargo.toml").exists() {
        bail!(
            "no Nargo.toml found in {} -- is this a Noir project?",
            project_dir.display()
        );
    }

    let source_file = find_source_file(&project_dir)?;

    eprintln!("compiling circuit...");
    nargo::check(&project_dir).context("circuit validation failed")?;
    eprintln!("circuit compiled successfully");

    let response = ipfs::add_file(ipfs_rpc_url, &source_file)
        .await
        .context("failed to upload circuit to IPFS")?;

    println!("{}", response.hash);

    let data = NewComplianceDefinitionData {
        cid: response.hash,
        file_name: response.name,
        file_path: source_file.display().to_string(),
        project_dir: project_dir.display().to_string(),
        ipfs_size: response.size,
    };

    let receipt = Receipt::new("new-compliance-definition", data);
    receipt.write_to(output)?;

    Ok(())
}
