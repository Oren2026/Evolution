//! Memory Graph — 記憶圖
//!
//! 持久化儲存所有已驗證的節點和呼叫鏈。
//!
//! 核心思想：
//! - 每個任務完成後，有效的呼叫鏈寫入記憶圖
//! - 新任務進來時，從記憶圖查詢是否已有驗證過的鏈
//! - 每次執行都是「從葉往上追溯」，而不是「從根往下執行」

use super::{ChainNode, Node, NodeCategory};
use std::collections::{HashMap, HashSet};

/// 記憶圖
///
/// 結構：
/// - `nodes` — 所有節點（ID → Node）
/// - `chains` — 已驗證的呼叫鏈（葉ID → ChainNode）
/// - `dependencies` — 依賴記錄（節點ID → 依賴的節點ID列表）
/// - `reverse_deps` — 反向依賴（被依賴的節點 → 哪些節點依賴它）
pub struct MemoryGraph {
    /// 所有節點
    nodes: HashMap<String, Box<dyn Node>>,
    /// 已驗證的呼叫鏈（key = leaf_id）
    chains: HashMap<String, ChainNode>,
    /// 依賴圖（節點 → 依賴節點列表）
    dependencies: HashMap<String, Vec<String>>,
    /// 反向依賴圖（節點 → 被誰依賴）
    reverse_dependencies: HashMap<String, HashSet<String>>,
    /// 命中統計（用於熱度分析）
    hit_count: HashMap<String, usize>,
}

impl MemoryGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            chains: HashMap::new(),
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            hit_count: HashMap::new(),
        }
    }

    // ===== 節點操作 =====

    /// 加入節點
    pub fn add_node<N: Node + 'static>(&mut self, node: N) {
        let id = node.id().to_string();

        // 維護依賴圖
        for dep_id in node.dependencies() {
            self.dependencies
                .entry(id.clone())
                .or_default()
                .push(dep_id.to_string());

            self.reverse_dependencies
                .entry(dep_id.to_string())
                .or_default()
                .insert(id.clone());
        }

        self.nodes.insert(id, Box::new(node));
    }

    /// 根據 ID 取得節點
    pub fn get_node(&mut self, id: &str) -> Option<&dyn Node> {
        let exists = self.nodes.contains_key(id);
        if exists {
            *self.hit_count.entry(id.to_string()).or_insert(0) += 1;
        }
        self.nodes.get(id).map(|b| b.as_ref())
    }

    /// 根據類別查詢節點
    pub fn get_nodes_by_category(&self, category: NodeCategory) -> Vec<&dyn Node> {
        self.nodes
            .values()
            .filter(|n| n.category() == category)
            .map(|b| b.as_ref())
            .collect()
    }

    /// 所有節點 ID 列表
    pub fn list_node_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    /// 節點是否存在
    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }

    /// 節點數量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // ===== 呼叫鏈操作 =====

    /// 注册呼叫鏈
    pub fn register_chain(&mut self, chain: ChainNode) {
        let leaf_id = chain.leaf_id.clone();
        self.chains.insert(leaf_id.clone(), chain);
        // 增加熱度
        *self.hit_count.entry(leaf_id).or_insert(0) += 10;
    }

    /// 查詢葉節點的已驗證呼叫鏈
    pub fn find_chain(&self, leaf_id: &str) -> Option<&ChainNode> {
        self.chains.get(leaf_id)
    }

    /// 所有已驗證的呼叫鏈
    pub fn all_chains(&self) -> Vec<&ChainNode> {
        self.chains.values().collect()
    }

    /// 呼叫鏈數量
    pub fn chain_count(&self) -> usize {
        self.chains.len()
    }

    // ===== 依賴圖操作 =====

    /// 獲取節點的直接依賴
    pub fn get_dependencies(&self, node_id: &str) -> Option<&Vec<String>> {
        self.dependencies.get(node_id)
    }

    /// 獲取哪些節點依賴某節點（反向查詢）
    pub fn get_reverse_dependencies(&self, node_id: &str) -> Option<&HashSet<String>> {
        self.reverse_dependencies.get(node_id)
    }

    /// 從葉往上遍歷完整依賴路徑（可用於 ChainDiscovery）
    pub fn trace_upwards(&self, leaf_id: &str) -> Vec<String> {
        let mut path = vec![leaf_id.to_string()];
        let mut current = leaf_id;

        while let Some(deps) = self.dependencies.get(current) {
            if let Some(first_dep) = deps.first() {
                path.push(first_dep.clone());
                current = first_dep;
            } else {
                break;
            }
        }

        path
    }

    // ===== 熱度統計 =====

    /// 取得熱度最高的 N 個節點
    pub fn hottest(&self, n: usize) -> Vec<(&str, usize)> {
        let mut sorted: Vec<(&str, usize)> = self
            .hit_count
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    /// 增加命中計數
    pub fn hit(&mut self, id: &str) {
        *self.hit_count.entry(id.to_string()).or_insert(0) += 1;
    }

    /// 取得命中計數
    pub fn get_hit_count(&self, id: &str) -> Option<usize> {
        self.hit_count.get(id).copied()
    }

    // ===== 持久化 =====

    /// 轉換成可序列化的格式
    pub fn to_persisted(&self) -> crate::storage::PersistedGraph {
        use crate::storage::PersistedGraph;
        let chains = self
            .chains
            .iter()
            .map(|(leaf_id, chain)| {
                (
                    leaf_id.clone(),
                    chain.path.clone(),
                    chain.verified,
                )
            })
            .collect::<Vec<_>>();
        let hits = self
            .hit_count
            .iter()
            .map(|(k, v)| (k.clone(), (*v) as u32))
            .collect::<Vec<_>>();
        PersistedGraph::from_chains_and_hits(chains, hits)
    }

    /// 從持久化格式還原
    pub fn from_persisted(pg: &crate::storage::PersistedGraph) -> Self {
        let mut graph = Self::new();
        for chain in &pg.chains {
            let mut c = ChainNode::new(&chain.leaf_id, chain.path.clone());
            if chain.verified {
                c.mark_verified();
            }
            graph.register_chain(c);
        }
        for (id, count) in &pg.hit_counts {
            graph.hit_count.insert(id.clone(), (*count) as usize);
        }
        graph
    }

    // ===== 清除操作 =====

    /// 清除所有資料（用於測試）
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.chains.clear();
        self.dependencies.clear();
        self.reverse_dependencies.clear();
        self.hit_count.clear();
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{Context, NodeResult, NodeCategory, Node};

    struct TestSkillNode {
        id: String,
        deps: Vec<String>,
    }

    impl TestSkillNode {
        fn new(id: &str, deps: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                deps: deps.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl Node for TestSkillNode {
        fn id(&self) -> &str {
            &self.id
        }

        fn dependencies(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }

        fn category(&self) -> NodeCategory {
            NodeCategory::Skill
        }

        fn execute(&self, _ctx: &Context) -> NodeResult {
            NodeResult::ok("ok")
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // ===== 基本操作測試 =====

    #[test]
    fn test_add_and_get_node() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("skill_a", vec![]));

        assert!(graph.has_node("skill_a"));
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node("skill_a").is_some());
    }

    #[test]
    fn test_get_nodes_by_category() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("skill_x", vec![]));
        graph.add_node(TestSkillNode::new("skill_y", vec![]));

        let skills = graph.get_nodes_by_category(NodeCategory::Skill);
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn test_list_node_ids() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("a", vec![]));
        graph.add_node(TestSkillNode::new("b", vec![]));

        let ids = graph.list_node_ids();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    // ===== 依賴圖測試 =====

    #[test]
    fn test_dependencies_tracking() {
        let mut graph = MemoryGraph::new();
        // chain: C → B → A
        graph.add_node(TestSkillNode::new("C", vec!["B"]));
        graph.add_node(TestSkillNode::new("B", vec!["A"]));
        graph.add_node(TestSkillNode::new("A", vec![]));

        let deps_c = graph.get_dependencies("C");
        assert!(deps_c.is_some());
        assert_eq!(deps_c.unwrap(), &vec!["B"]);

        let rdeps_a = graph.get_reverse_dependencies("A");
        assert!(rdeps_a.is_some());
        assert!(rdeps_a.unwrap().contains(&"B".to_string()));
    }

    #[test]
    fn test_trace_upwards() {
        let mut graph = MemoryGraph::new();
        // chain: leaf → parent → grandparent
        graph.add_node(TestSkillNode::new("leaf", vec!["parent"]));
        graph.add_node(TestSkillNode::new("parent", vec!["grandparent"]));
        graph.add_node(TestSkillNode::new("grandparent", vec![]));

        let path = graph.trace_upwards("leaf");
        assert_eq!(path, vec!["leaf", "parent", "grandparent"]);
    }

    // ===== 呼叫鏈測試 =====

    #[test]
    fn test_register_and_find_chain() {
        let mut graph = MemoryGraph::new();
        let chain = ChainNode::new("leaf", vec!["leaf".to_string(), "mid".to_string(), "root".to_string()]);
        graph.register_chain(chain);

        let found = graph.find_chain("leaf");
        assert!(found.is_some());
        assert_eq!(found.unwrap().path.len(), 3);
    }

    #[test]
    fn test_chain_count() {
        let mut graph = MemoryGraph::new();
        graph.register_chain(ChainNode::new("leaf1", vec!["leaf1".to_string()]));
        graph.register_chain(ChainNode::new("leaf2", vec!["leaf2".to_string()]));

        assert_eq!(graph.chain_count(), 2);
        assert_eq!(graph.all_chains().len(), 2);
    }

    // ===== 熱度測試 =====

    #[test]
    fn test_hottest() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("cold", vec![]));
        graph.add_node(TestSkillNode::new("warm", vec![]));
        graph.add_node(TestSkillNode::new("hot", vec![]));

        graph.hit("hot");
        graph.hit("hot");
        graph.hit("warm");

        let hottest = graph.hottest(2);
        assert_eq!(hottest[0].0, "hot");
        assert_eq!(hottest[0].1, 2);
        assert_eq!(hottest[1].0, "warm");
    }

    #[test]
    fn test_hit_count() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("test_node", vec![]));

        // 第一次 get_node 會增加 count
        graph.get_node("test_node");
        graph.get_node("test_node");
        graph.get_node("test_node");

        let hottest = graph.hottest(1);
        assert_eq!(hottest[0].1, 3);
    }

    // ===== 清除測試 =====

    #[test]
    fn test_clear() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("x", vec![]));
        graph.register_chain(ChainNode::new("leaf", vec!["leaf".to_string()]));

        graph.clear();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.chain_count(), 0);
        assert!(graph.list_node_ids().is_empty());
    }

    // ===== 邊界條件測試 =====

    #[test]
    fn test_nonexistent_node() {
        let mut graph = MemoryGraph::new();
        assert!(graph.get_node("nonexistent").is_none());
        assert!(!graph.has_node("nonexistent"));
        assert!(graph.get_dependencies("nonexistent").is_none());
        assert!(graph.find_chain("nonexistent").is_none());
    }

    #[test]
    fn test_trace_upwards_no_deps() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestSkillNode::new("orphan", vec![]));

        let path = graph.trace_upwards("orphan");
        assert_eq!(path, vec!["orphan"]);
    }
}