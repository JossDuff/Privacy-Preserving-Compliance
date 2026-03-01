use anyhow::{Context, Result};
use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddResponse {
    pub name: String,
    pub hash: String,
    pub size: String,
}

// Upload a file to IPFS
pub async fn add_file(ipfs_rpc_url: &str, file_path: &Path) -> Result<AddResponse> {
    let file_name = file_path
        .file_name()
        .context("file path has no file name")?
        .to_string_lossy()
        .to_string();

    let file_bytes = tokio::fs::read(file_path)
        .await
        .with_context(|| format!("failed to read file: {}", file_path.display()))?;

    let part = multipart::Part::bytes(file_bytes).file_name(file_name);
    let form = multipart::Form::new().part("file", part);

    let url = format!("{}/api/v0/add", ipfs_rpc_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .with_context(|| format!(
            "failed to upload {} to IPFS at {url} -- is the IPFS daemon running?",
            file_path.display()
        ))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "IPFS add failed for {} (HTTP {status} from {url}): {body}",
            file_path.display()
        );
    }

    let add_response: AddResponse = response
        .json()
        .await
        .with_context(|| format!(
            "failed to parse IPFS add response from {url} for {}",
            file_path.display()
        ))?;

    Ok(add_response)
}
