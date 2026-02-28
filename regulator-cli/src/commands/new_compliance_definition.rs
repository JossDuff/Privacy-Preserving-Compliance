use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::ipfs;

pub async fn run(file: PathBuf, ipfs_rpc_url: &str) -> Result<()> {
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

    Ok(())
}
