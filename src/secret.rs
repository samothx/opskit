//! Reading API keys and tokens: environment override, then a file (systemd credentials aware).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// `$env_var` (if set and non-empty) wins. Otherwise the file is read; a relative name is looked
/// up in `$CREDENTIALS_DIRECTORY` (systemd `LoadCredential=`) first.
pub fn read(env_var: &str, file: &Path) -> Result<String> {
    if let Ok(k) = std::env::var(env_var)
        && !k.trim().is_empty()
    {
        return Ok(k.trim().to_string());
    }
    let path = resolve(file);
    let key = std::fs::read_to_string(&path).with_context(|| format!("reading secret from {}", path.display()))?;
    let key = key.trim();
    if key.is_empty() {
        bail!("secret file {} is empty", path.display());
    }
    Ok(key.to_string())
}

pub fn resolve(file: &Path) -> PathBuf {
    if file.is_relative()
        && let Ok(dir) = std::env::var("CREDENTIALS_DIRECTORY")
    {
        return Path::new(&dir).join(file);
    }
    file.to_path_buf()
}
