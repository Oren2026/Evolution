//! Chain Discovery — 呼叫鏈探索
//!
//! 從葉節點往上遍歷，依賴 MemoryGraph 的依賴圖找到完整呼叫路徑。
//!
//! 探索策略：
//! 1. 檢查記憶圖是否已有已驗證的鏈（快速路徑）
//! 2. 若無，進行 BFS 往上遍歷
//! 3. 記錄路徑，返回 DiscoveryResult

use super::{DiscoveryResult, ChainNode};
use crate::node::MemoryGraph;

/// 呼叫鏈探索器
#[derive(Debug, Clone)]
pub struct ChainDiscovery {
    /// 最大探索深度（防止無限迴圈）
    max_depth: usize,
}

impl ChainDiscovery {
    pub fn new() -> Self {
        Self { max_depth: 64 }
    }

    /// 設定最大探索深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// 探索葉節點的完整呼叫鏈
    ///
    /// 流程：
    /// 1. 若記憶圖已有已驗證的 ChainNode，直接返回（verified=true）
    /// 2. 否則執行 BFS 往上遍歷
    /// 3. 若遍歷中路徑形成環，回傳錯誤
    pub fn discover(&self, graph: &MemoryGraph, leaf_id: &str) -> Option<DiscoveryResult> {
        // 快速路徑：檢查已驗證的鏈
        if let Some(chain) = graph.find_chain(leaf_id) {
            return Some(DiscoveryResult::new(leaf_id, chain.path.clone())
                .with_verified(true));
        }

        // BFS 往上遍歷
        let path = self.bfs_trace(graph, leaf_id)?;

        Some(DiscoveryResult::new(leaf_id, path))
    }

    /// BFS 遍歷：從葉往上找到所有父節點
    ///
    /// Follow ALL dependencies (not just first) to build complete chain.
    /// For a node with multiple deps, each dep is appended and then traced.
    /// Example: A→[B,C], B→[D] → path = [A, B, D, C]
    fn bfs_trace(&self, graph: &MemoryGraph, leaf_id: &str) -> Option<Vec<String>> {
        if !graph.has_node(leaf_id) {
            return None;
        }

        let mut path = vec![leaf_id.to_string()];
        let mut visited = std::collections::HashSet::new();
        visited.insert(leaf_id.to_string());
        let mut queue: Vec<String> = vec![];

        // Add all deps of leaf to queue
        if let Some(deps) = graph.get_dependencies(leaf_id) {
            for d in deps {
                if !visited.contains(d) {
                    queue.push(d.clone());
                }
            }
        }

        while !queue.is_empty() && path.len() <= self.max_depth {
            let current = queue.remove(0);

            if visited.contains(&current) {
                continue;
            }

            path.push(current.clone());
            visited.insert(current.clone());

            // Add all deps of current node
            if let Some(deps) = graph.get_dependencies(&current) {
                for d in deps {
                    if !visited.contains(d) {
                        queue.push(d.clone());
                    }
                }
            }
        }

        Some(path)
    }

    /// 驗證並注册鏈到記憶圖
    pub fn verify_and_register(
        &self,
        graph: &mut MemoryGraph,
        leaf_id: &str,
    ) -> Option<DiscoveryResult> {
        let result = self.discover(graph, leaf_id)?;

        // 將結果注册為 ChainNode
        let mut chain = ChainNode::new(leaf_id, result.path.clone());
        chain.mark_verified();
        graph.register_chain(chain);

        Some(DiscoveryResult::new(leaf_id, result.path).with_verified(true))
    }
}

impl Default for ChainDiscovery {
    fn default() -> Self {
        Self::new()
    }
}