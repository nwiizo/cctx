use anyhow::{Context, Result, bail};
use std::fs;
use std::io::Write;
use std::path::Path;

pub(crate) fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "-"
        || name.starts_with('.')
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
    {
        bail!("invalid name {name:?}: use a visible filename without path separators");
    }
    Ok(())
}

pub(crate) fn read_settings(path: &Path) -> Result<String> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    validate_settings(&content)?;
    Ok(content)
}

pub(crate) fn validate_settings(content: &str) -> Result<()> {
    let json: serde_json::Value = serde_json::from_str(content).context("Invalid settings JSON")?;
    if !json.is_object() {
        bail!("Settings must be a JSON object");
    }
    Ok(())
}

/// Replace one file without exposing partially written JSON. New files are private.
pub(crate) fn atomic_write(path: &Path, content: impl AsRef<[u8]>) -> Result<()> {
    let parent = path.parent().context("File has no parent directory")?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(content.as_ref())?;
    file.as_file().sync_all()?;
    file.persist(path)
        .with_context(|| format!("Failed to replace {}", path.display()))?;
    Ok(())
}
