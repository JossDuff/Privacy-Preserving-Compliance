use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
struct CastSendJson {
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
}

pub struct CastSendOutput {
    pub transaction_hash: String,
}

/// Run `cast send` to call a function on a deployed contract.
pub fn send(
    rpc_url: &str,
    private_key: &str,
    to: &str,
    sig: &str,
    args: &[&str],
) -> Result<CastSendOutput> {
    let mut cmd = Command::new("cast");
    cmd.args([
        "send", "--json", "--rpc-url", rpc_url, "--private-key", private_key, to, sig,
    ]);
    cmd.args(args);

    let output = cmd
        .output()
        .context("failed to run cast send -- is foundry installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cast send failed:\n{stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: CastSendJson =
        serde_json::from_str(&stdout).context("failed to parse cast send output")?;

    Ok(CastSendOutput {
        transaction_hash: parsed.transaction_hash,
    })
}
