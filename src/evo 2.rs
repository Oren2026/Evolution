//! EvolutionOS — 統一的 AI 作業系統入口
//!
//! 整合 Node、Executor、ModelDispatcher、Storage，讓外部可以一句话启动系统。

use crate::model::{ModelDispatcher, ModelRequest, ModelResponse, DispatchError};
use crate::node::{Context, MemoryGraph, NodeResult};
use crate::runtime::Executor;
use crate::storage::{JsonStorage, PersistedGraph, Storage, StorageError};

/// EvolutionOS 主結構
pub struct EvolutionOS {
    graph: MemoryGraph,
    executor: Executor,
    model_dispatcher: Box<dyn ModelDispatcher>,
    storage: JsonStorage,
    auto_persist: bool,
    persist_threshold: u32,
    execute_count: u32,
}

impl EvolutionOS {
    /// 建立新的 EvolutionOS
    pub fn new(dispatcher: Box<dyn ModelDispatcher>, storage: JsonStorage) -> Self {
        Self {
            graph: MemoryGraph::new(),
            executor: Executor::new(),
            model_dispatcher: dispatcher,
            storage,
            auto_persist: true,
            persist_threshold: 10,
            execute_count: 0,
        }
    }

    /// 載入或建立 — 若磁碟有資料則還原，否則新建
    pub fn load_or_create(dispatcher: Box<dyn ModelDispatcher>, storage: JsonStorage) -> Result<Self, LoadError> {
        if storage.exists() {
            let pg = storage.load().map_err(LoadError::Storage)?;
            let graph = MemoryGraph::from_persisted(&pg);
            Ok(Self {
                graph,
                executor: Executor::new(),
                model_dispatcher: dispatcher,
                storage,
                auto_persist: true,
                persist_threshold: 10,
                execute_count: 0,
            })
        } else {
            Ok(Self::new(dispatcher, storage))
        }
    }

    /// 執行葉節點
    pub fn execute(&mut self, leaf_id: &str, _input: &str) -> Result<NodeResult, ExecuteError> {
        self.execute_count += 1;

        // 執行
        let result = self
            .executor
            .execute_or_discover(&mut self.graph, leaf_id, _input);

        // 增加命中計數
        self.graph.hit(leaf_id);

        // 自動快照
        if self.auto_persist && self.execute_count % self.persist_threshold == 0 {
            self.persist().map_err(ExecuteError::Persist)?;
        }

        Ok(result)
    }

    /// 手動寫入磁碟
    pub fn persist(&self) -> Result<(), StorageError> {
        let pg = self.graph.to_persisted();
        self.storage.save(&pg)
    }

    /// 健康檢查
    pub fn health_check(&self) -> HealthStatus {
        let pg = if self.storage.exists() {
            self.storage.load().ok()
        } else {
            None
        };
        HealthStatus {
            graph_nodes: self.graph.node_count(),
            stored_chains: pg.as_ref().map(|p| p.chains.len()).unwrap_or(0),
            model_available: self.model_dispatcher.health_check(),
            last_persisted: pg.map(|p| p.version),
            execute_count: self.execute_count,
        }
    }

    /// AI 模型呼叫
    pub fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError> {
        self.model_dispatcher.dispatch(req)
    }

    /// 取得圖形參照（唯讀）
    pub fn graph(&self) -> &MemoryGraph {
        &self.graph
    }
}

/// 執行錯誤
#[derive(Debug)]
pub enum ExecuteError {
    Persist(StorageError),
}

/// 載入錯誤
#[derive(Debug)]
pub enum LoadError {
    Storage(StorageError),
}

/// 健康狀態
#[derive(Debug)]
pub struct HealthStatus {
    pub graph_nodes: usize,
    pub stored_chains: usize,
    pub model_available: bool,
    pub last_persisted: Option<String>,
    pub execute_count: u32,
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteError::Persist(s) => write!(f, "persist error: {}", s),
        }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Storage(s) => write!(f, "storage error: {}", s),
        }
    }
}

impl std::error::Error for ExecuteError {}
impl std::error::Error for LoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockDispatcher {
        available: bool,
    }
    impl ModelDispatcher for MockDispatcher {
        fn dispatch(&self, _req: ModelRequest) -> Result<ModelResponse, DispatchError> {
            Ok(ModelResponse {
                content: "mock response".into(),
                model: "mock".into(),
                tokens_used: 5,
            })
        }
        fn available_models(&self) -> Vec<String> {
            if self.available { vec!["mock".into()] } else { vec![] }
        }
    }

    #[test]
    fn test_new_and_health_check() {
        let storage = JsonStorage::new(&std::env::temp_dir().join("evolution_os_test_health.json").to_string_lossy());
        let dispatcher = Box::new(MockDispatcher { available: true });
        let os = EvolutionOS::new(dispatcher, storage);

        let health = os.health_check();
        assert_eq!(health.graph_nodes, 0);
        assert!(health.model_available);
    }

    #[test]
    fn test_persist_and_load() {
        let path = std::env::temp_dir().join("evolution_os_test_persist.json");
        let storage = JsonStorage::new(&path.to_string_lossy());
        let dispatcher = Box::new(MockDispatcher { available: true });

        let os = EvolutionOS::new(dispatcher, storage);
        os.persist().unwrap();

        // 用 load_or_create 還原
        let storage2 = JsonStorage::new(&path.to_string_lossy());
        let dispatcher2 = Box::new(MockDispatcher { available: true });
        let os2 = EvolutionOS::load_or_create(dispatcher2, storage2).unwrap();

        assert_eq!(os2.health_check().graph_nodes, 0);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_dispatch() {
        let storage = JsonStorage::new(&std::env::temp_dir().join("evolution_os_test_dispatch.json").to_string_lossy());
        let dispatcher = Box::new(MockDispatcher { available: true });
        let os = EvolutionOS::new(dispatcher, storage);

        let resp = os.dispatch(ModelRequest::new("mock", "hello")).unwrap();
        assert_eq!(resp.content, "mock response");
    }
}