//! Atomic JSON state files (`<dir>/<name>`): a missing file is the default state.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn load<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// Writes `<path>.tmp` and renames it over `path`, creating the directory if needed.
pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = Path::new(&tmp);
    std::fs::write(tmp, serde_json::to_vec_pretty(value)?).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn roundtrip_and_missing() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sub/state.json");
        let empty: BTreeMap<String, i64> = super::load(&p).unwrap();
        assert!(empty.is_empty());
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), 1_i64);
        super::save(&p, &m).unwrap();
        assert_eq!(super::load::<BTreeMap<String, i64>>(&p).unwrap(), m);
        assert!(!dir.path().join("sub/state.json.tmp").exists());
    }
}
