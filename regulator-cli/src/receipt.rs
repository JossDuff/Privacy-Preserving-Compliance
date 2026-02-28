use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct Receipt<T: Serialize> {
    pub command: String,
    pub timestamp: String,
    pub data: T,
}

impl<T: Serialize> Receipt<T> {
    pub fn new(command: &str, data: T) -> Self {
        Self {
            command: command.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data,
        }
    }

    /// Write the receipt as JSON to a file inside `dir`, creating the directory if needed.
    /// The filename is `<command>-<timestamp>.json` (filesystem-safe).
    pub fn write_to_dir(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create receipts directory {}", dir.display()))?;

        let safe_ts = Utc::now().format("%Y%m%dT%H%M%S").to_string();
        let filename = format!("{}-{}.json", self.command, safe_ts);
        let path = dir.join(&filename);

        let json =
            serde_json::to_string_pretty(self).context("failed to serialize receipt")?;
        std::fs::write(&path, &json)
            .with_context(|| format!("failed to write receipt to {}", path.display()))?;
        eprintln!("receipt written to {}", path.display());
        Ok(())
    }
}
