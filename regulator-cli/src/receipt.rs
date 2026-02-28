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

    pub fn write_to(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("failed to serialize receipt")?;
        std::fs::write(path, &json)
            .with_context(|| format!("failed to write receipt to {}", path.display()))?;
        eprintln!("receipt written to {}", path.display());
        Ok(())
    }
}
