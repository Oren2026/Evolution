//! JsonStorage — 將圖形序列化為 JSON 檔案
//!
//! 預設路徑：`~/.evolution_os/graph.json`

use super::{PersistedGraph, Storage, StorageError};
use std::fs;
use std::path::PathBuf;

/// JSON 檔案儲存
pub struct JsonStorage {
    path: PathBuf,
}

impl JsonStorage {
    pub fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    pub fn default_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".evolution_os")
            .join("graph.json")
    }

    pub fn with_default_path() -> Self {
        Self::new(&Self::default_path().to_string_lossy())
    }

    fn ensure_dir(&self) -> Result<(), StorageError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

impl Storage for JsonStorage {
    fn save(&self, data: &PersistedGraph) -> Result<(), StorageError> {
        self.ensure_dir()?;
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&self.path, json)
            .map_err(|e| StorageError::Io(format!("failed to write {}: {}", self.path.display(), e)))?;
        Ok(())
    }

    fn load(&self) -> Result<PersistedGraph, StorageError> {
        let content =
            fs::read_to_string(&self.path).map_err(|e| StorageError::Load(e.to_string()))?;
        serde_json::from_str(&content).map_err(|e| StorageError::Load(format!(
            "failed to parse {}: {}",
            self.path.display(),
            e
        )))
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_json_storage_roundtrip() {
        let tmp = std::env::temp_dir().join("evolution_os_test.json");
        let storage = JsonStorage::new(&tmp.to_string_lossy());

        let graph = PersistedGraph::from_chains_and_hits(
            vec![(
                "leaf1".into(),
                vec!["leaf1".into(), "mid".into(), "root".into()],
                true,
            )],
            vec![("n1".into(), 3u32)],
        );

        storage.save(&graph).unwrap();
        assert!(storage.exists());

        let loaded = storage.load().unwrap();
        assert_eq!(loaded.chains.len(), 1);
        assert_eq!(loaded.chains[0].leaf_id, "leaf1");
        assert!(loaded.chains[0].verified);
        assert_eq!(loaded.hit_counts.len(), 1);

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_load_nonexistent() {
        let storage = JsonStorage::new("/nonexistent/path/graph.json");
        assert!(!storage.exists());
        assert!(storage.load().is_err());
    }
}