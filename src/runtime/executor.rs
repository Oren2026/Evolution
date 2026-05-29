//! Executor — 執行器
//!
//! 執行已解析的呼叫鏈：
//! 1. ChainDiscovery 探索完整路徑（從葉往上）
//! 2. 反轉路徑（變成從根到葉的執行順序）
//! 3. 按順序執行每個節點，上下文一路往下傳

use crate::chain::ChainDiscovery;
use crate::node::{Context, MemoryGraph, NodeResult};

/// 執行器
#[derive(Debug, Clone)]
pub struct Executor {
    /// 最大執行深度（防止無限迴圈）
    max_depth: usize,
}

impl Executor {
    pub fn new() -> Self {
        Self { max_depth: 64 }
    }

    /// 設定最大執行深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// 執行單一節點（不追蹤依賴鏈，直接執行）
    ///
    /// 直接執行指定的節點，用 initial_input 作為其輸入。
    /// 若要自動追蹤依賴鏈，請用 execute_or_discover。
    pub fn execute_node(
        &self,
        graph: &mut MemoryGraph,
        node_id: &str,
        initial_input: &str,
    ) -> NodeResult {
        let mut ctx = Context::new(node_id);
        ctx.insert("input", initial_input);

        let node = match graph.get_node(node_id) {
            Some(n) => n,
            None => return NodeResult::err(&format!("node not found: {}", node_id)),
        };

        let result = node.execute(&ctx);
        graph.hit(node_id);
        result
    }

    /// 執行葉節點的完整呼叫鏈
    ///
    /// 流程：
    /// 1. ChainDiscovery 探索完整路徑（從葉往上）
    /// 2. 反轉路徑（從根到葉，變成執行順序）
    /// 3. 按順序執行每個節點，上一節點的輸出傳給下一節點
    pub fn execute(
        &self,
        graph: &mut MemoryGraph,
        leaf_id: &str,
        initial_input: &str,
    ) -> NodeResult {
        let discovery = ChainDiscovery::new();

        // 探索路徑（從葉往上）
        let path = match discovery.discover(graph, leaf_id) {
            Some(p) => p.path,
            None => {
                return NodeResult::err(&format!("cannot discover path for leaf: {}", leaf_id))
            }
        };

        // 反轉：從根到葉（變成執行順序）
        let forward_path: Vec<String> = path.into_iter().rev().collect();

        if forward_path.len() > self.max_depth {
            return NodeResult::err(&format!(
                "execution path too deep: {} (max: {})",
                forward_path.len(),
                self.max_depth
            ));
        }

        // 建立執行上下文
        let mut ctx = Context::new(leaf_id);
        let mut last_result = NodeResult::ok("");

        // 按順序執行（從根到葉），輸出傳給下一節點
        for (i, node_id) in forward_path.iter().enumerate() {
            ctx.push_parent(node_id);

            // 每個節點的輸入是上一節點的輸出（第一節點用 initial_input）
            let node_input = if i == 0 { initial_input } else { &last_result.output };
            ctx.insert("input", node_input);

            if let Some(node) = graph.get_node(node_id) {
                last_result = node.execute(&ctx);
                if !last_result.success {
                    // 執行失敗，停止鏈
                    return last_result;
                }
                // 更新記憶圖命中計數
                graph.hit(node_id);
            } else {
                return NodeResult::err(&format!("node not found during execution: {}", node_id));
            }
        }

        last_result
    }

    /// 嘗試執行，若無已驗證路徑則先探索再執行
    pub fn execute_or_discover(
        &self,
        graph: &mut MemoryGraph,
        leaf_id: &str,
        initial_input: &str,
    ) -> NodeResult {
        // 先確保有已驗證的路徑
        let discovery = ChainDiscovery::new();
        if discovery.discover(graph, leaf_id).is_none() {
            return NodeResult::err(&format!("cannot discover path for: {}", leaf_id));
        }

        // 驗證並注册路徑
        discovery.verify_and_register(graph, leaf_id);

        // 執行
        self.execute(graph, leaf_id, initial_input)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{Node, NodeCategory};

    // 測試用節點
    struct TestNode {
        id: String,
        deps: Vec<String>,
        result: NodeResult,
    }

    impl TestNode {
        fn new_ok(id: &str, deps: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                deps: deps.iter().map(|s| s.to_string()).collect(),
                result: NodeResult::ok("ok"),
            }
        }

        fn new_err(id: &str, deps: Vec<&str>, msg: &str) -> Self {
            Self {
                id: id.to_string(),
                deps: deps.iter().map(|s| s.to_string()).collect(),
                result: NodeResult::err(msg),
            }
        }
    }

    impl Node for TestNode {
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
            self.result.clone()
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn make_graph(chain: Vec<(&str, Vec<&str>)>) -> MemoryGraph {
        let mut graph = MemoryGraph::new();
        for (id, deps) in chain {
            graph.add_node(TestNode::new_ok(id, deps));
        }
        graph
    }

    #[test]
    fn test_executor_new() {
        let exec = Executor::new();
        assert_eq!(exec.max_depth, 64);
    }

    #[test]
    fn test_executor_with_max_depth() {
        let exec = Executor::new().with_max_depth(10);
        assert_eq!(exec.max_depth, 10);
    }

    #[test]
    fn test_execute_simple_chain() {
        let mut graph = make_graph(vec![("A", vec![]), ("B", vec!["A"]), ("C", vec!["B"])]);

        let exec = Executor::new();
        let result = exec.execute(&mut graph, "C", "");

        assert!(result.success);
    }

    #[test]
    fn test_execute_node_not_found() {
        let mut graph = make_graph(vec![("A", vec![]), ("B", vec!["A"])]);

        let exec = Executor::new();
        let result = exec.execute(&mut graph, "ghost", "");

        assert!(!result.success);
        assert!(result.error.unwrap().contains("cannot discover path"));
    }

    #[test]
    fn test_execute_failure_stops_chain() {
        let mut graph = MemoryGraph::new();
        graph.add_node(TestNode::new_ok("A", vec![]));
        graph.add_node(TestNode::new_err("B", vec!["A"], "intentional failure"));
        graph.add_node(TestNode::new_ok("C", vec!["B"]));

        let exec = Executor::new();
        let result = exec.execute(&mut graph, "C", "");

        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "intentional failure");
    }

    #[test]
    fn test_execute_empty_graph() {
        let mut graph = MemoryGraph::new();
        let exec = Executor::new();
        let result = exec.execute(&mut graph, "nonexistent", "");

        assert!(!result.success);
    }

    #[test]
    fn test_execute_or_discover_new_chain() {
        let mut graph = make_graph(vec![("X", vec![]), ("Y", vec!["X"])]);

        let exec = Executor::new();
        let result = exec.execute_or_discover(&mut graph, "Y", "");

        assert!(result.success);
        // 確認已注册
        assert!(graph.find_chain("Y").is_some());
    }

    #[test]
    fn test_execute_depth_limit() {
        let mut graph = MemoryGraph::new();
        // 建立一個很深的鏈（65個節點，超過 max_depth=64）
        for i in 0..65 {
            let node_id = format!("n{}", i);
            let deps: Vec<String> = if i > 0 {
                vec![format!("n{}", i - 1)]
            } else {
                vec![]
            };
            let deps_refs: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
            graph.add_node(TestNode::new_ok(&node_id, deps_refs));
        }

        let exec = Executor::new();
        let result = exec.execute(&mut graph, "n64", "");

        assert!(!result.success);
        assert!(result.error.unwrap().contains("too deep"));
    }
}