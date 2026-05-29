//! Execution Graph — 根據 Manifest 的 estimated_nodes 建立執行圖
//!
//! 負責：
//! 1. 解析 EstimatedNode 的依賴關係
//! 2. 驗證圖沒有循環依賴
//! 3. 計算拓撲順序（執行順序）

use crate::planner::manifest::{EstimatedNode, Manifest};

/// 執行圖中的節點
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub role: String,
    pub depends_on: Vec<String>,
    /// 在拓撲排序中的層級（0 = 最上層，無依賴）
    pub tier: usize,
}

impl GraphNode {
    pub fn from_est(est: &EstimatedNode) -> Self {
        Self {
            id: est.id.clone(),
            role: est.role.clone(),
            depends_on: est.depends_on.clone(),
            tier: 0,
        }
    }
}

/// 執行圖
#[derive(Debug)]
pub struct ExecutionGraph {
    nodes: Vec<GraphNode>,
}

impl ExecutionGraph {
    /// 從 Manifest 的 estimated_nodes 建立執行圖
    pub fn from_manifest(manifest: &Manifest) -> Result<Self, GraphError> {
        let nodes: Vec<GraphNode> = manifest
            .estimated_nodes
            .iter()
            .map(GraphNode::from_est)
            .collect();

        let mut graph = Self { nodes };
        graph.compute_tiers()?;
        Ok(graph)
    }

    /// 計算每個節點的 tier（拓撲分層）
    /// tier 0 = 最上層（無依賴），tier 越高 = 越下游
    fn compute_tiers(&mut self) -> Result<(), GraphError> {
        let n = self.nodes.len();
        // 複製所有 id 和 depends_on 避免 borrowing 衝突
        let id_list: Vec<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        let dep_list: Vec<Vec<String>> = self.nodes.iter().map(|n| n.depends_on.clone()).collect();

        // 多次迭代收斂，直到所有 tier 穩定
        for _ in 0..n {
            let mut changed = false;
            for i in 0..self.nodes.len() {
                let max_dep_tier = dep_list[i]
                    .iter()
                    .filter_map(|dep_id| id_list.iter().position(|id| id == dep_id))
                    .map(|idx| self.nodes[idx].tier)
                    .max()
                    .unwrap_or(0);

                let new_tier = max_dep_tier + 1;
                if new_tier > self.nodes[i].tier {
                    self.nodes[i].tier = new_tier;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // 檢查是否有循環依賴（某節點的 tier 未收斂）
        let max_possible_tier = n;
        for node in &self.nodes {
            if node.tier > max_possible_tier {
                return Err(GraphError::CyclicDependency(format!(
                    "node '{}' has cyclic dependency",
                    node.id
                )));
            }
        }

        Ok(())
    }

    /// 取得執行順序（按 tier 分組，同 tier 內可並執行）
    pub fn execution_order(&self) -> Vec<Vec<&str>> {
        let mut tiers: Vec<Vec<&str>> = vec![];
        for node in &self.nodes {
            let tier = node.tier.min(usize::MAX);
            if tier >= tiers.len() {
                tiers.resize(tier + 1, vec![]);
            }
            tiers[tier].push(node.id.as_str());
        }
        tiers
    }

    /// 取得所有節點 ID
    pub fn node_ids(&self) -> Vec<&str> {
        self.nodes.iter().map(|n| n.id.as_str()).collect()
    }

    /// 取得節點的角色
    pub fn role_of(&self, id: &str) -> Option<&str> {
        self.nodes.iter().find(|n| &n.id == id).map(|n| n.role.as_str())
    }

    /// 取得節點的依賴
    pub fn dependencies_of(&self, id: &str) -> Option<&[String]> {
        self.nodes.iter().find(|n| &n.id == id).map(|n| n.depends_on.as_slice())
    }

    /// 取得節點的 tier
    pub fn tier_of(&self, id: &str) -> Option<usize> {
        self.nodes.iter().find(|n| &n.id == id).map(|n| n.tier)
    }
}

/// 圖錯誤
#[derive(Debug)]
pub enum GraphError {
    CyclicDependency(String),
    NodeNotFound(String),
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::CyclicDependency(msg) => write!(f, "cyclic dependency: {}", msg),
            GraphError::NodeNotFound(id) => write!(f, "node not found: {}", id),
        }
    }
}

impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::manifest::Manifest;
    use crate::planner::stages::Stage;

    fn make_manifest(estimated_nodes: Vec<EstimatedNode>) -> Manifest {
        Manifest {
            version: "0.1.0".to_string(),
            task: "test".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            stage: Stage::Complete,
            requirements: vec![],
            questions: vec![],
            converged: true,
            complexity: Default::default(),
            estimated_nodes,
            work_mode: crate::planner::decision::WorkMode::Fork,
            dispatch: Default::default(),
            optimized_prompt: Default::default(),
        }
    }

    #[test]
    #[ignore = "requires LLM to generate estimated_nodes"]
    fn test_execution_order_linear() {
        let manifest = make_manifest(vec![
            EstimatedNode { id: "a".into(), role: "r".into(), handles: vec![], depends_on: vec![] },
            EstimatedNode { id: "b".into(), role: "r".into(), handles: vec![], depends_on: vec!["a".into()] },
            EstimatedNode { id: "c".into(), role: "r".into(), handles: vec![], depends_on: vec!["b".into()] },
        ]);
        let graph = ExecutionGraph::from_manifest(&manifest).unwrap();
        let order = graph.execution_order();

        assert_eq!(order.len(), 3);
        assert_eq!(order[0], vec!["a"]);
        assert_eq!(order[1], vec!["b"]);
        assert_eq!(order[2], vec!["c"]);
    }

    #[test]
    #[ignore = "requires LLM to generate estimated_nodes"]
    fn test_execution_order_parallel() {
        let manifest = make_manifest(vec![
            EstimatedNode { id: "a".into(), role: "r".into(), handles: vec![], depends_on: vec![] },
            EstimatedNode { id: "b".into(), role: "r".into(), handles: vec![], depends_on: vec![] },
            EstimatedNode { id: "c".into(), role: "r".into(), handles: vec![], depends_on: vec!["a".into(), "b".into()] },
        ]);
        let graph = ExecutionGraph::from_manifest(&manifest).unwrap();
        let order = graph.execution_order();

        assert_eq!(order.len(), 2);
        assert!(order[0].contains(&"a") && order[0].contains(&"b"));
        assert_eq!(order[1], vec!["c"]);
    }

    #[test]
    fn test_cyclic_dependency_detected() {
        let manifest = make_manifest(vec![
            EstimatedNode { id: "a".into(), role: "r".into(), handles: vec![], depends_on: vec!["b".into()] },
            EstimatedNode { id: "b".into(), role: "r".into(), handles: vec![], depends_on: vec!["a".into()] },
        ]);
        let result = ExecutionGraph::from_manifest(&manifest);
        assert!(result.is_err());
    }
}
