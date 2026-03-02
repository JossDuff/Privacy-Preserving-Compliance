use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

const ETHERSCAN_V2_API: &str = "https://api.etherscan.io/v2/api";
const POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_POLL_ATTEMPTS: u32 = 20;
const SUBMIT_RETRIES: u32 = 3;
const SUBMIT_RETRY_DELAY: Duration = Duration::from_secs(10);

/// Optional Etherscan/block-explorer verification settings.
#[derive(Clone, Default)]
pub struct VerifyArgs {
    pub etherscan_api_key: Option<String>,
    pub verifier_url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct EtherscanResponse<T> {
    status: String,
    result: T,
}

impl<T> EtherscanResponse<T> {
    fn is_ok(&self) -> bool {
        self.status == "1"
    }
}

/// Outcome of a contract verification attempt.
#[derive(Debug)]
pub enum VerificationOutcome {
    Verified,
    AlreadyVerified,
    Failed(String),
    Skipped,
}

impl std::fmt::Display for VerificationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified => write!(f, "verified"),
            Self::AlreadyVerified => write!(f, "already_verified"),
            Self::Failed(reason) => write!(f, "failed: {reason}"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

/// Map a chain ID to its block explorer base URL for human-readable links.
fn explorer_url(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "https://etherscan.io",
        11155111 => "https://sepolia.etherscan.io",
        8453 => "https://basescan.org",
        42161 => "https://arbiscan.io",
        137 => "https://polygonscan.com",
        10 => "https://optimistic.etherscan.io",
        _ => "https://etherscan.io",
    }
}

/// Map a chain ID to a human-readable network name.
pub fn network_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "Mainnet",
        11155111 => "Sepolia",
        8453 => "Base",
        84532 => "Base Sepolia",
        42161 => "Arbitrum One",
        421614 => "Arbitrum Sepolia",
        10 => "Optimism",
        11155420 => "Optimism Sepolia",
        137 => "Polygon",
        _ => "unknown network",
    }
}

/// Build Solidity Standard JSON Input from a forge project's source files and artifact metadata.
///
/// Reads all source files referenced in the artifact metadata and reconstructs the
/// compiler input that Etherscan needs to reproduce the bytecode.
fn build_standard_json_input(project_dir: &Path, artifact_path: &Path) -> Result<(String, String)> {
    let artifact_bytes = std::fs::read(artifact_path)
        .with_context(|| format!("failed to read artifact: {}", artifact_path.display()))?;
    let artifact: serde_json::Value = serde_json::from_slice(&artifact_bytes)?;

    // Extract metadata — forge stores it as a JSON string in "rawMetadata"
    let metadata: serde_json::Value = if let Some(raw) = artifact.get("rawMetadata") {
        let s = raw.as_str().context("rawMetadata is not a string")?;
        serde_json::from_str(s)?
    } else if let Some(m) = artifact.get("metadata") {
        if let Some(s) = m.as_str() {
            serde_json::from_str(s)?
        } else {
            m.clone()
        }
    } else {
        bail!("no metadata found in artifact {}", artifact_path.display());
    };

    // Compiler version — Etherscan expects "v0.8.28+commit.xyz"
    let version = metadata
        .pointer("/compiler/version")
        .and_then(|v| v.as_str())
        .context("no compiler.version in artifact metadata")?;
    let compiler_version = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    };

    // Settings from metadata (optimizer, evmVersion, remappings, etc.)
    let meta_settings = metadata
        .get("settings")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let mut settings = serde_json::Map::new();
    if let Some(obj) = meta_settings.as_object() {
        for (k, v) in obj {
            if k == "compilationTarget" {
                continue;
            }
            settings.insert(k.clone(), v.clone());
        }
    }
    settings
        .entry("outputSelection")
        .or_insert(serde_json::json!({
            "*": { "*": ["abi", "evm.bytecode", "evm.deployedBytecode"] }
        }));

    // Read source files listed in metadata
    let source_keys = metadata
        .get("sources")
        .and_then(|s| s.as_object())
        .context("no sources in artifact metadata")?;

    let mut sources = serde_json::Map::new();
    for path in source_keys.keys() {
        let full_path = project_dir.join(path);
        let content = std::fs::read_to_string(&full_path)
            .with_context(|| format!("failed to read source: {}", full_path.display()))?;
        sources.insert(path.clone(), serde_json::json!({ "content": content }));
    }

    let standard_json = serde_json::json!({
        "language": "Solidity",
        "sources": sources,
        "settings": settings,
    });

    let json_str =
        serde_json::to_string(&standard_json).context("failed to serialize standard JSON input")?;

    Ok((json_str, compiler_version))
}

async fn submit_verification(
    client: &reqwest::Client,
    base_url: &str,
    chain_id: u64,
    api_key: &str,
    contract_address: &str,
    standard_json_input: &str,
    contract_name: &str,
    compiler_version: &str,
    constructor_args: &str,
) -> Result<String> {
    let chain_id_str = chain_id.to_string();
    let form_params = [
        ("module", "contract"),
        ("action", "verifysourcecode"),
        ("contractaddress", contract_address),
        ("sourceCode", standard_json_input),
        ("codeformat", "solidity-standard-json-input"),
        ("contractname", contract_name),
        ("compilerversion", compiler_version),
        ("constructorArguments", constructor_args),
    ];

    let resp = client
        .post(base_url)
        .query(&[("chainid", &chain_id_str), ("apikey", &api_key.to_string())])
        .form(&form_params)
        .send()
        .await
        .context("failed to send verification request to Etherscan")?
        .json::<EtherscanResponse<String>>()
        .await
        .context("failed to parse Etherscan verification response")?;

    if !resp.is_ok() {
        bail!("Etherscan verification submission failed: {}", resp.result);
    }

    Ok(resp.result)
}

async fn poll_status(
    client: &reqwest::Client,
    base_url: &str,
    chain_id: u64,
    api_key: &str,
    guid: &str,
    indent: &str,
) -> Result<VerificationOutcome> {
    let chain_id_str = chain_id.to_string();

    for attempt in 1..=MAX_POLL_ATTEMPTS {
        sleep(POLL_INTERVAL).await;

        let resp = client
            .get(base_url)
            .query(&[
                ("chainid", chain_id_str.as_str()),
                ("module", "contract"),
                ("action", "checkverifystatus"),
                ("guid", guid),
                ("apikey", api_key),
            ])
            .send()
            .await
            .context("failed to poll Etherscan verification status")?
            .json::<EtherscanResponse<String>>()
            .await
            .context("failed to parse Etherscan status response")?;

        eprintln!(
            "{indent}  verification check ({attempt}/{MAX_POLL_ATTEMPTS}): {}",
            resp.result
        );

        match resp.result.as_str() {
            "Pass - Verified" => return Ok(VerificationOutcome::Verified),
            "Already Verified" => return Ok(VerificationOutcome::AlreadyVerified),
            "Pending in queue" => continue,
            other => return Ok(VerificationOutcome::Failed(other.to_string())),
        }
    }

    Ok(VerificationOutcome::Failed(format!(
        "timed out after {MAX_POLL_ATTEMPTS} attempts"
    )))
}

/// Verify a deployed contract on Etherscan (or compatible explorer) using the v2 API.
///
/// Returns the verification outcome. If no API key is configured, returns `Skipped`.
/// Prints progress to stderr and the final explorer link to stdout.
pub async fn verify_contract(
    project_dir: &Path,
    artifact_path: &Path,
    chain_id: u64,
    contract_address: &str,
    contract_name: &str,
    constructor_args: Option<&str>,
    verify: &VerifyArgs,
    indent: &str,
) -> Result<VerificationOutcome> {
    let api_key = match verify
        .etherscan_api_key
        .as_deref()
        .filter(|k| !k.is_empty())
    {
        Some(key) => key,
        None => {
            eprintln!("{indent}no Etherscan API key provided, skipping verification");
            return Ok(VerificationOutcome::Skipped);
        }
    };

    let base_url = verify
        .verifier_url
        .as_deref()
        .filter(|u| !u.is_empty())
        .unwrap_or(ETHERSCAN_V2_API);

    eprintln!("{indent}verifying {contract_address} on chain {chain_id}...");

    let (standard_json, compiler_version) =
        build_standard_json_input(project_dir, artifact_path)
            .context("failed to build standard JSON input for verification")?;

    let client = reqwest::Client::new();
    let constructor_args = constructor_args.unwrap_or("");

    let mut guid = None;
    for attempt in 1..=SUBMIT_RETRIES {
        eprintln!("{indent}  submission attempt {attempt}/{SUBMIT_RETRIES}...");
        match submit_verification(
            &client,
            base_url,
            chain_id,
            api_key,
            contract_address,
            &standard_json,
            contract_name,
            &compiler_version,
            constructor_args,
        )
        .await
        {
            Ok(g) => {
                guid = Some(g);
                break;
            }
            Err(e) => {
                if attempt < SUBMIT_RETRIES {
                    eprintln!(
                        "{indent}  attempt {attempt} failed: {e:#}, retrying in {}s...",
                        SUBMIT_RETRY_DELAY.as_secs()
                    );
                    sleep(SUBMIT_RETRY_DELAY).await;
                } else {
                    eprintln!("{indent}  all {SUBMIT_RETRIES} attempts failed: {e:#}");
                    return Ok(VerificationOutcome::Failed(format!("{e:#}")));
                }
            }
        }
    }
    let guid = guid.expect("guid set if loop didn't return");

    eprintln!("{indent}  submitted (guid: {guid}), polling for result...");

    let outcome = poll_status(&client, base_url, chain_id, api_key, &guid, indent).await?;

    let explorer = explorer_url(chain_id);
    match &outcome {
        VerificationOutcome::Verified => {
            eprintln!("{indent}  verified: {explorer}/address/{contract_address}#code");
        }
        VerificationOutcome::AlreadyVerified => {
            eprintln!("{indent}  already verified: {explorer}/address/{contract_address}#code");
        }
        VerificationOutcome::Failed(reason) => {
            eprintln!("{indent}  verification failed: {reason}");
        }
        VerificationOutcome::Skipped => {}
    }

    Ok(outcome)
}
