//! Storage — 持久化儲存介面
//!
//! 讓 MemoryGraph 可以持久化到磁碟，重啟後可還原。

mod json_storage;

pub use json_storage::JsonStorage;

/// 儲存錯誤
#[derive(Debug, Clone)]
pub enum StorageError {
    Io(String),
    Serialization(String),
    Load(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(s) => write!(f, "IO error: {}", s),
            StorageError::Serialization(s) => write!(f, "serialization error: {}", s),
            StorageError::Load(s) => write!(f, "load error: {}", s),
        }
    }
}

impl std::error::Error for StorageError {}

/// 儲存 trait — 所有持久化實作必須實現
pub trait Storage: Send + Sync {
    /// 儲存到磁碟
    fn save(&self, data: &PersistedGraph) -> Result<(), StorageError>;
    /// 從磁碟載入
    fn load(&self) -> Result<PersistedGraph, StorageError>;
    /// 檔案是否存在
    fn exists(&self) -> bool;
}

/// 可序列化的圖形資料（不含 dyn Node，只有 Chain + Metadata）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedGraph {
    pub version: String,
    pub chains: Vec<PersistedChain>,
    pub hit_counts: Vec<(String, u32)>,
}

impl PersistedGraph {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            chains: Vec::new(),
            hit_counts: Vec::new(),
        }
    }

    pub fn from_chains_and_hits(
        chains: impl IntoIterator<Item = (String, Vec<String>, bool)>,
        hit_counts: impl IntoIterator<Item = (String, u32)>,
    ) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            chains: chains
                .into_iter()
                .map(|(leaf_id, path, verified)| PersistedChain {
                    leaf_id,
                    path,
                    verified,
                })
                .collect(),
            hit_counts: hit_counts.into_iter().collect(),
        }
    }
}

impl Default for PersistedGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// 可序列化的 ChainNode
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedChain {
    pub leaf_id: String,
    pub path: Vec<String>,
    pub verified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persisted_graph_roundtrip() {
        let graph = PersistedGraph::from_chains_and_hits(
            vec![
                ("leaf_a".into(), vec!["leaf_a".into(), "b".into(), "c".into()], true),
                ("leaf_b".into(), vec!["leaf_b".into(), "c".into()], false),
            ],
            vec![("a".into(), 5u32), ("b".into(), 10u32)],
        );

        let json = serde_json::to_string(&graph).unwrap();
        let loaded: PersistedGraph = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.chains.len(), 2);
        assert_eq!(loaded.chains[0].leaf_id, "leaf_a");
        assert_eq!(loaded.chains[0].path.len(), 3);
        assert!(loaded.chains[0].verified);
        assert_eq!(loaded.hit_counts.len(), 2);
    }

    #[test]
    fn test_persisted_graph_empty() {
        let graph = PersistedGraph::new();
        let json = serde_json::to_string(&graph).unwrap();
        let loaded: PersistedGraph = serde_json::from_str(&json).unwrap();
        assert!(loaded.chains.is_empty());
        assert!(loaded.hit_counts.is_empty());
    }

    #[test]
    fn test_storage_error_display() {
        let e = StorageError::Io("file not found".into());
        assert!(e.to_string().contains("file not found"));
    }
}