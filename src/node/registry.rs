//! Node Registry — 節點注册表
//!
//! 管理所有節點的注册、查詢、移除。

use super::{Node, NodeCategory};
use std::any::Any;
use std::collections::HashMap;

/// 節點注册表
// Debug 需要手動實作，因為 dyn Node 沒有 Debug
pub struct NodeRegistry {
    nodes: HashMap<String, Box<dyn Node>>,
}

impl std::fmt::Debug for NodeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRegistry")
            .field("nodes", &self.nodes.len())
            .finish()
    }
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// 注册新節點
    pub fn register<N: Node + 'static>(&mut self, node: N) {
        let id = node.id().to_string();
        self.nodes.insert(id, Box::new(node));
    }

    /// 根據 ID 獲取節點（可變借用）
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Box<dyn Node>> {
        self.nodes.get_mut(id)
    }

    /// 根據 ID 獲取節點（不可變借用）
    pub fn get(&self, id: &str) -> Option<&dyn Node> {
        self.nodes.get(id).map(|b| b.as_ref())
    }

    /// 根據類別獲取所有節點
    pub fn get_by_category(&self, category: NodeCategory) -> Vec<&dyn Node> {
        self.nodes
            .values()
            .filter(|n| n.category() == category)
            .map(|b| b.as_ref())
            .collect()
    }

    /// 列出所有節點 ID
    pub fn list_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    /// 節點數量
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// 是否為空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 節點Handle — 封裝了對註冊節點的引用
#[derive(Debug)]
pub struct NodeHandle<'a> {
    id: &'a str,
    registry: &'a NodeRegistry,
}

impl<'a> NodeHandle<'a> {
    pub fn new(id: &'a str, registry: &'a NodeRegistry) -> Self {
        Self { id, registry }
    }

    pub fn get(&self) -> Option<&dyn Node> {
        self.registry.get(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{Context, NodeResult, NodeCategory};

    struct DummyNode {
        id: String,
        deps: Vec<String>,
        cat: NodeCategory,
    }

    impl DummyNode {
        fn new(id: &str, deps: Vec<&str>, cat: NodeCategory) -> Self {
            Self {
                id: id.to_string(),
                deps: deps.iter().map(|s| s.to_string()).collect(),
                cat,
            }
        }
    }

    impl Node for DummyNode {
        fn id(&self) -> &str {
            &self.id
        }

        fn dependencies(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }

        fn category(&self) -> NodeCategory {
            self.cat
        }

        fn execute(&self, _ctx: &Context) -> NodeResult {
            NodeResult::ok("dummy output")
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = NodeRegistry::new();
        reg.register(DummyNode::new("node_a", vec![], NodeCategory::Skill));

        let node = reg.get("node_a");
        assert!(node.is_some());
        assert_eq!(node.unwrap().id(), "node_a");
    }

    #[test]
    fn test_registry_get_mut() {
        let mut reg = NodeRegistry::new();
        reg.register(DummyNode::new("node_b", vec![], NodeCategory::Context));

        let node = reg.get_mut("node_b");
        assert!(node.is_some());
    }

    #[test]
    fn test_registry_get_by_category() {
        let mut reg = NodeRegistry::new();
        reg.register(DummyNode::new("skill_1", vec![], NodeCategory::Skill));
        reg.register(DummyNode::new("skill_2", vec![], NodeCategory::Skill));
        reg.register(DummyNode::new("ctx_1", vec![], NodeCategory::Context));

        let skills = reg.get_by_category(NodeCategory::Skill);
        assert_eq!(skills.len(), 2);

        let contexts = reg.get_by_category(NodeCategory::Context);
        assert_eq!(contexts.len(), 1);
    }

    #[test]
    fn test_registry_list_ids() {
        let mut reg = NodeRegistry::new();
        reg.register(DummyNode::new("x", vec![], NodeCategory::Skill));
        reg.register(DummyNode::new("y", vec![], NodeCategory::Skill));

        let ids = reg.list_ids();
        assert!(ids.contains(&"x"));
        assert!(ids.contains(&"y"));
    }

    #[test]
    fn test_registry_len() {
        let mut reg = NodeRegistry::new();
        assert_eq!(reg.len(), 0);

        reg.register(DummyNode::new("a", vec![], NodeCategory::Skill));
        assert_eq!(reg.len(), 1);

        reg.register(DummyNode::new("b", vec![], NodeCategory::Skill));
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn test_registry_is_empty() {
        let reg = NodeRegistry::new();
        assert!(reg.is_empty());

        let mut reg = NodeRegistry::new();
        reg.register(DummyNode::new("a", vec![], NodeCategory::Skill));
        assert!(!reg.is_empty());
    }

    #[test]
    fn test_node_handle() {
        let reg = NodeRegistry::new();
        let handle = NodeHandle::new("nonexistent", &reg);
        assert!(handle.get().is_none());
    }
}