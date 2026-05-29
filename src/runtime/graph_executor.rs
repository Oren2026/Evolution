//! GraphExecutor — 根據 tier 順序執行編譯後的節點圖
//!
//! 這是 Compiler 的配套執行器：
//! 1. 接收 Compiler 輸出的 MemoryGraph（含所有 LLMNode）
//! 2. 根據 tier 順序執行（同一 tier 可並行）
//! 3. 每個節點的輸出傳給下游依賴節點

use crate::compiler::ExecutionGraph;
use crate::node::{Context, DynSkillNode, MemoryGraph, NodeResult};

/// 基於拓撲 tier 的執行器
#[derive(Debug, Clone)]
pub struct GraphExecutor {
    max_tier: usize,
}

impl GraphExecutor {
    pub fn new() -> Self {
        Self { max_tier: 16 }
    }

    /// 執行完整的編譯後圖
    /// 
    /// 流程：
    /// 1. 取得 ExecutionGraph 的執行順序（按 tier 分組）
    /// 2. 依序執行每個 tier 的節點
    /// 3. 收集每個節點的輸出，供下游使用
    pub fn execute(&self, graph: &mut MemoryGraph, exec_graph: &ExecutionGraph) -> Vec<NodeResult> {
        let mut results = vec![];
        let mut tier_outputs: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for tier_nodes in exec_graph.execution_order() {
            let mut tier_results = vec![];

            for node_id in tier_nodes {
                let node_id_str = (*node_id).to_string();

                // 建立上下文：包含任務輸入 + 上游輸出
                let mut ctx = Context::new(&node_id_str);
                ctx.insert("tier", &format!("{}", exec_graph.tier_of(&node_id_str).unwrap_or(0)));

                // 收集上游依賴節點的輸出
                let upstream_context: String = exec_graph
                    .dependencies_of(&node_id_str)
                    .map(|deps| {
                        deps.iter()
                            .filter_map(|dep_id| tier_outputs.get(dep_id).cloned())
                            .collect::<Vec<_>>()
                            .join("\n---\n")
                    })
                    .unwrap_or_default();

                if !upstream_context.is_empty() {
                    ctx.insert("upstream_output", &upstream_context);
                }

                // 執行節點（先取值，避免 borrowing 衝突）
                let result = {
                    let node_opt = graph.get_node(&node_id_str);
                    match node_opt {
                        Some(node) => {
                            let mut ctx = Context::new(&node_id_str);
                            ctx.insert("tier", &format!("{}", exec_graph.tier_of(&node_id_str).unwrap_or(0)));
                            if !upstream_context.is_empty() {
                                ctx.insert("upstream_output", &upstream_context);
                            }
                            node.execute(&ctx)
                        }
                        None => NodeResult::err(&format!("node not found: {}", node_id_str)),
                    }
                };

                // 記錄輸出（clone result，避免 borrowing 衝突）
                if result.success {
                    tier_outputs.insert(node_id_str.clone(), result.output.clone());
                }

                // 更新 hit
                graph.hit(&node_id_str);
                tier_results.push(result);
            }

            results.extend(tier_results);
        }

        results
    }

    /// 執行單一節點（debug 用）
    pub fn execute_node(&self, graph: &mut MemoryGraph, node_id: &str, input: &str) -> NodeResult {
        // 先取值，避免 borrowing 衝突
        let node_opt = graph.get_node(node_id);

        let result = node_opt
            .map(|node| {
                let mut ctx = Context::new(node_id);
                ctx.insert("input", input);
                node.execute(&ctx)
            })
            .unwrap_or_else(|| NodeResult::err(&format!("node not found: {}", node_id)));

        graph.hit(node_id);
        result
    }
}

impl Default for GraphExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::ExecutionGraph;
    use crate::node::{Node, NodeCategory};
    use crate::planner::manifest::Manifest;

    struct DummyNode {
        id: String,
        deps: Vec<String>,
    }

    impl Node for DummyNode {
        fn id(&self) -> &str { &self.id }
        fn dependencies(&self) -> Vec<&str> { self.deps.iter().map(|s| s.as_str()).collect() }
        fn category(&self) -> NodeCategory { NodeCategory::Skill }
        fn execute(&self, ctx: &Context) -> NodeResult {
            NodeResult::ok(&format!("executed: {}", ctx.get("input").unwrap_or("none")))
        }
        fn as_any(&self) -> &dyn std::any::Any { self }
    }

    #[test]
    fn test_graph_executor_simple() {
        let exec = GraphExecutor::new();
        let mut graph = MemoryGraph::new();
        graph.add_node(DummyNode { id: "a".into(), deps: vec![] });
        graph.add_node(DummyNode { id: "b".into(), deps: vec!["a".into()] });

        // Build manifest with estimated_nodes so ExecutionGraph has nodes
        let manifest = {
            use crate::planner::manifest::Manifest;
            use crate::planner::stages::Stage;
            Manifest {
                version: "0.1.0".to_string(),
                task: "test".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                stage: Stage::Complete,
                requirements: vec![],
                questions: vec![],
                converged: true,
                complexity: Default::default(),
                estimated_nodes: vec![
                    crate::planner::manifest::EstimatedNode {
                        id: "a".into(), role: "r".into(), handles: vec![], depends_on: vec![],
                    },
                    crate::planner::manifest::EstimatedNode {
                        id: "b".into(), role: "r".into(), handles: vec![], depends_on: vec!["a".into()],
                    },
                ],
                work_mode: Default::default(),
                dispatch: Default::default(),
                optimized_prompt: Default::default(),
            }
        };
        let exec_graph = ExecutionGraph::from_manifest(&manifest).unwrap();

        let results = exec.execute(&mut graph, &exec_graph);
        assert!(!results.is_empty());
    }
}