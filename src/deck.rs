use anyhow::{Context, Result, anyhow};
use std::fs;

pub fn validate(path: &str) -> Result<()> {
    fs::metadata(path).with_context(|| anyhow!("Path '{}' is invalid", path))?;
    Ok(())
}
