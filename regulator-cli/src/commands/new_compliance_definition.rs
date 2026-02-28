use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::PathBuf;

use crate::ipfs;
use crate::receipt::Receipt;

#[derive(Debug, Serialize)]
pub struct NewComplianceDefinitionData {
    pub cid: String,
    pub file_name: String,
    pub file_path: String,
    pub ipfs_size: String,
}

pub async fn run(file: PathBuf, ipfs_rpc_url: &str, output: &PathBuf) -> Result<()> {
    if !file.exists() {
        bail!("file not found: {}", file.display());
    }

    match file.extension().and_then(|e| e.to_str()) {
        Some("nr") => {}
        Some(ext) => bail!("expected a .nr file, got .{ext}"),
        None => bail!("file has no extension, expected .nr"),
    }

    let response = ipfs::add_file(ipfs_rpc_url, &file)
        .await
        .context("failed to upload circuit to IPFS")?;

    println!("{}", response.hash);

    let data = NewComplianceDefinitionData {
        cid: response.hash,
        file_name: response.name,
        file_path: file.display().to_string(),
        ipfs_size: response.size,
    };

    let receipt = Receipt::new("new-compliance-definition", data);
    receipt.write_to(output)?;

    Ok(())
}
