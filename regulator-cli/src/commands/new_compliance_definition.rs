use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::forge;
use crate::forge::VerifyArgs;
use crate::receipt::Receipt;

#[derive(Debug, Serialize)]
pub struct NewComplianceDefinitionData {
    pub contract_address: String,
    pub transaction_hash: String,
    pub regulator: String,
    pub rpc_url: String,
}

pub async fn run(
    rpc_url: &str,
    private_key: &str,
    regulator: &str,
    contract_dir: &Path,
    receipts_dir: &Path,
    verify: &VerifyArgs,
) -> Result<()> {
    eprintln!("compiling contracts...");
    forge::build(contract_dir)?;
    eprintln!("contracts compiled successfully");

    eprintln!("deploying ComplianceDefinition...");
    let result = forge::create(
        contract_dir,
        rpc_url,
        private_key,
        "src/ComplianceDefinition.sol:ComplianceDefinition",
        &[regulator],
        verify,
    )?;
    eprintln!("deployed successfully");

    println!("{}", result.deployed_to);

    let data = NewComplianceDefinitionData {
        contract_address: result.deployed_to,
        transaction_hash: result.transaction_hash,
        regulator: regulator.to_string(),
        rpc_url: rpc_url.to_string(),
    };

    let receipt = Receipt::new("new-compliance-definition", data);
    receipt.write_to_dir(receipts_dir)?;

    Ok(())
}
