//! Chain — 呼叫鏈追蹤模組
//!
//! 核心：從葉節點往上探索，依賴 `use` 宣告追蹤實際呼叫關係。

mod discovery;

pub use discovery::ChainDiscovery;

use crate::node::ChainNode;

/// 探索結果
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// 葉節點 ID
    pub leaf_id: String,
    /// 從葉到根的完整路徑
    pub path: Vec<String>,
    /// 路徑是否已驗證
    pub verified: bool,
    /// 探索深度
    pub depth: usize,
}

impl DiscoveryResult {
    pub fn new(leaf_id: &str, path: Vec<String>) -> Self {
        let depth = path.len().saturating_sub(1);
        Self {
            leaf_id: leaf_id.to_string(),
            path,
            verified: false,
            depth,
        }
    }

    pub fn with_verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{Context, NodeResult, NodeCategory, Node};
    use crate::node::MemoryGraph;

    struct DummySkillNode {
        id: String,
        deps: Vec<String>,
    }

    impl DummySkillNode {
        fn new(id: &str, deps: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                deps: deps.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl Node for DummySkillNode {
        fn id(&self) -> &str { &self.id }
        fn dependencies(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }
        fn category(&self) -> NodeCategory { NodeCategory::Skill }
        fn execute(&self, _ctx: &Context) -> NodeResult { NodeResult::ok("ok") }
        fn as_any(&self) -> &dyn std::any::Any { self }
    }

    // ===== DiscoveryResult Tests =====

    #[test]
    fn test_discovery_result_new() {
        let result = DiscoveryResult::new(
            "leaf",
            vec!["leaf".to_string(), "parent".to_string(), "root".to_string()],
        );
        assert_eq!(result.leaf_id, "leaf");
        assert_eq!(result.path.len(), 3);
        assert_eq!(result.depth, 2);
        assert!(!result.verified);
    }

    #[test]
    fn test_discovery_result_with_verified() {
        let result = DiscoveryResult::new("leaf", vec!["leaf".to_string()])
            .with_verified(true);
        assert!(result.verified);
    }

    // ===== ChainDiscovery Tests =====

    #[test]
    fn test_chain_discovery_simple() {
        let mut graph = MemoryGraph::new();
        graph.add_node(DummySkillNode::new("A", vec![]));
        graph.add_node(DummySkillNode::new("B", vec!["A"]));
        graph.add_node(DummySkillNode::new("C", vec!["B"]));

        let discovery = ChainDiscovery::new();
        let result = discovery.discover(&graph, "C").unwrap();

        assert_eq!(result.leaf_id, "C");
        assert_eq!(result.path, vec!["C", "B", "A"]);
        assert_eq!(result.depth, 2);
    }

    #[test]
    fn test_chain_discovery_no_deps() {
        let mut graph = MemoryGraph::new();
        graph.add_node(DummySkillNode::new("lonely", vec![]));

        let discovery = ChainDiscovery::new();
        let result = discovery.discover(&graph, "lonely").unwrap();

        assert_eq!(result.leaf_id, "lonely");
        assert_eq!(result.path, vec!["lonely"]);
        assert_eq!(result.depth, 0);
    }

    #[test]
    fn test_chain_discovery_nonexistent() {
        let graph = MemoryGraph::new();
        let discovery = ChainDiscovery::new();
        let result = discovery.discover(&graph, "ghost");
        assert!(result.is_none());
    }

    #[test]
    fn test_chain_discovery_with_existing_chain() {
        let mut graph = MemoryGraph::new();
        graph.add_node(DummySkillNode::new("X", vec![]));
        graph.add_node(DummySkillNode::new("Y", vec!["X"]));
        graph.add_node(DummySkillNode::new("Z", vec!["Y"]));

        // 預先注册一個已驗證的鏈
        let mut chain = ChainNode::new("Z", vec!["Z".to_string(), "Y".to_string(), "X".to_string()]);
        chain.mark_verified();
        graph.register_chain(chain);

        let discovery = ChainDiscovery::new();
        let result = discovery.discover(&graph, "Z").unwrap();

        assert!(result.verified);
    }

    #[test]
    fn test_chain_discovery_multi_path() {
        let mut graph = MemoryGraph::new();
        // A 是根，B 和 C 都依賴 A
        graph.add_node(DummySkillNode::new("A", vec![]));
        graph.add_node(DummySkillNode::new("B", vec!["A"]));
        graph.add_node(DummySkillNode::new("C", vec!["A"]));
        // D 同時依賴 B 和 C
        graph.add_node(DummySkillNode::new("D", vec!["B", "C"]));

        let discovery = ChainDiscovery::new();
        let result = discovery.discover(&graph, "D").unwrap();

        // 只取第一個依賴（BFS）
        assert_eq!(result.path[0], "D");
        assert_eq!(result.path[1], "B");
    }
}