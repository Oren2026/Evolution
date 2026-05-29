//! Compiler — PlannerManifest → 可執行節點圖
//!
//! 這是 Evolution OS v0.3 的核心：將 Planner 的規劃結果轉化為實際可執行的節點圖。
//!
//! 流程：
//! 1. Planner 輸出 PlannerManifest（分工藍圖）
//! 2. Compiler 根據 Manifest 建立對應的 LLMNode
//! 3. ExecutionGraph 分析依賴關係，計算執行順序
//! 4. Executor 按拓撲順序執行，最終輸出實作結果

pub mod execution_graph;
pub mod node_factory;

pub use execution_graph::{ExecutionGraph, GraphError};
pub use node_factory::LLMNode;

use crate::model::OllamaBackend;
use crate::node::{DynSkillNode, MemoryGraph};
use crate::planner::manifest::Manifest;
use crate::runtime::Executor;
use std::sync::{Arc, Mutex};

/// Compiler — 將 PlannerManifest 編譯為可執行的節點圖
pub struct Compiler {
    executor: Executor,
    backend: Arc<Mutex<Box<dyn crate::model::ModelDispatcher>>>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            executor: Executor::new(),
            backend: Arc::new(Mutex::new(Box::new(OllamaBackend::new()) as Box<dyn crate::model::ModelDispatcher>)),
        }
    }

    /// 從 PlannerManifest 編譯並執行
    ///
    /// 流程：
    /// 1. 根據 estimated_nodes 建立 LLMNode
    /// 2. 建立 ExecutionGraph 分析依賴
    /// 3. 註冊進 MemoryGraph
    /// 4. 按拓撲順序執行
    pub fn compile_and_execute(&self, manifest: &Manifest) -> CompilerResult {
        use crate::planner::decision::WorkMode;

        // Solo 模式：直接用 optimized_prompt 執行單一 LLMNode
        if manifest.work_mode == WorkMode::Solo {
            return self.execute_solo(manifest);
        }

        // Fork 模式：建立多節點圖
        self.execute_fork(manifest)
    }

    /// Solo 模式：單一節點處理簡單任務
    fn execute_solo(&self, manifest: &Manifest) -> CompilerResult {
        let mut graph = MemoryGraph::new();
        let mut node = node_factory::LLMNode::new(
            &crate::planner::manifest::EstimatedNode {
                id: "solo".to_string(),
                role: "軟體工程師".to_string(),
                handles: vec![],
                depends_on: vec![],
            },
            self.backend.clone(),
            &manifest.task,
        );

        let skill_node = DynSkillNode::new(Box::new(node));
        graph.add_node(skill_node);

        let result = self.executor.execute_node(&mut graph, "solo", "");

        CompilerResult {
            success: result.success,
            node_outputs: vec![result],
            graph,
        }
    }

    /// Fork 模式：多節點分工處理複雜任務
    fn execute_fork(&self, manifest: &Manifest) -> CompilerResult {
        // 建立執行圖
        let exec_graph = match ExecutionGraph::from_manifest(manifest) {
            Ok(g) => g,
            Err(e) => {
                return CompilerResult {
                    success: false,
                    node_outputs: vec![],
                    graph: MemoryGraph::new(),
                };
            }
        };

        // 建立節點圖
        let mut memory_graph = MemoryGraph::new();
        let mut outputs: Vec<crate::node::NodeResult> = vec![];

        // 按 tier 順序執行（同一 tier 可並行，這裡用順序模拟）
        let order = exec_graph.execution_order();
        for tier_nodes in order {
            // 收集同 tier 節點的輸出
            let mut tier_outputs: Vec<(String, String)> = vec![];

            for node_id in tier_nodes {
                // 找出對應的 EstimatedNode
                let est = manifest
                    .estimated_nodes
                    .iter()
                    .find(|e| &e.id == node_id)
                    .expect("node not found in manifest");

                // 根據依賯節點的輸出構造 context
                let context = est
                    .depends_on
                    .iter()
                    .map(|dep_id| {
                        tier_outputs
                            .iter()
                            .find(|(id, _)| id == dep_id)
                            .map(|(_, out)| out.as_str())
                            .unwrap_or("")
                    })
                    .collect::<Vec<_>>()
                    .join("\n---\n");

                // 建立 LLMNode
                let llm_node = node_factory::LLMNode::new(est, self.backend.clone(), &manifest.task);
                let skill_node = DynSkillNode::new(Box::new(llm_node));
                memory_graph.add_node(skill_node);

                // 執行
                let result = self.executor.execute_node(&mut memory_graph, node_id, &context);
                outputs.push(result.clone());

                // 收集輸出供同 tier 其他節點使用
                tier_outputs.push((node_id.to_string(), result.output.clone()));
            }
        }

        CompilerResult {
            success: outputs.iter().all(|o| o.success),
            node_outputs: outputs,
            graph: memory_graph,
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 編譯結果
pub struct CompilerResult {
    pub success: bool,
    pub node_outputs: Vec<crate::node::NodeResult>,
    pub graph: MemoryGraph,
}

/// 編譯錯誤
#[derive(Debug)]
pub enum CompilerError {
    GraphError(String),
    ExecutionError(String),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::GraphError(msg) => write!(f, "graph error: {}", msg),
            CompilerError::ExecutionError(msg) => write!(f, "execution error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::manifest::Manifest;
    use crate::planner::stages::Stage;

    fn make_solo_manifest() -> Manifest {
        Manifest {
            version: "0.1.0".to_string(),
            task: "寫一個網頁計數器".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            stage: Stage::Complete,
            requirements: vec![],
            questions: vec![],
            converged: true,
            complexity: Default::default(),
            estimated_nodes: vec![],
            work_mode: crate::planner::decision::WorkMode::Solo,
            dispatch: Default::default(),
            optimized_prompt: Default::default(),
        }
    }

    #[test]
    fn test_compiler_new() {
        let _c = Compiler::new();
    }

    #[test]
    fn test_solo_manifest_structure() {
        let m = make_solo_manifest();
        assert_eq!(m.work_mode, crate::planner::decision::WorkMode::Solo);
    }
}
