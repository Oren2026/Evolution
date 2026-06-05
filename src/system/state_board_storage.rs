//! StateBoardStorage — 狀態表持久化
//!
//! 將 StateBoard 寫入 `~/.evolution/state_board.json`

use crate::system::state_board::{StateBoard, StateBoardSnapshot, TaskStage};
use crate::storage::StorageError;
use std::fs;
use std::path::PathBuf;

/// StateBoard 專用的 JSON 儲存（不走 Storage trait，直接用專用方法）
pub struct StateBoardStorage {
    path: PathBuf,
}

impl StateBoardStorage {
    pub fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    pub fn default_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".evolution")
            .join("state_board.json")
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

    /// 儲存 StateBoard 快照
    pub fn save(&self, board: &StateBoard) -> Result<(), StorageError> {
        self.ensure_dir()?;
        let snap = board.snapshot();
        let json =
            serde_json::to_string_pretty(&snap).map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(&self.path, json)
            .map_err(|e| StorageError::Io(format!("failed to write {}: {}", self.path.display(), e)))?;
        Ok(())
    }

    /// 載入 StateBoard 快照
    pub fn load(&self) -> Result<StateBoard, StorageError> {
        let content =
            fs::read_to_string(&self.path).map_err(|e| StorageError::Load(e.to_string()))?;
        let snap: StateBoardSnapshot =
            serde_json::from_str(&content).map_err(|e| StorageError::Load(format!(
                "failed to parse {}: {}",
                self.path.display(),
                e
            )))?;
        Ok(StateBoard::from_snapshot(snap))
    }

    /// 檢查檔案是否存在
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// 刪除 StateBoard
    pub fn remove(&self) -> Result<(), StorageError> {
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|e| StorageError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_state_board_storage_roundtrip() {
        let tmp = std::env::temp_dir().join("evolution_state_board_test.json");
        let storage = StateBoardStorage::new(&tmp.to_string_lossy());

        // 建立 board 並寫入
        let mut board = StateBoard::new();
        board.create_task("task-a".into());
        let id2 = board.create_task("task-b".into());
        board.update_stage(&id2, TaskStage::Done, None);

        storage.save(&board).unwrap();
        assert!(storage.exists());

        // 讀回
        let restored = storage.load().unwrap();
        assert_eq!(restored.list_tasks().len(), 2);
        assert_eq!(restored.list_tasks()[1].stage, TaskStage::Done);
        assert_eq!(restored.undelivered_events().len(), 1);

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_load_nonexistent() {
        let storage = StateBoardStorage::new("/nonexistent/board.json");
        assert!(!storage.exists());
        assert!(storage.load().is_err());
    }
}