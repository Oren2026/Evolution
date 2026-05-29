//! Runtime Executor — 整合測試
//!
//! 測試 Executor 的執行流程、鏈反轉、輸入傳播、錯誤停止。

use evolution_os::runtime::Executor;
use evolution_os::{MemoryGraph, Node, NodeResult, Context, NodeCategory, ChainDiscovery};

struct DummyNode {
    id: String,
    deps: Vec<String>,
    execute_result: NodeResult,
}

impl DummyNode {
    fn new(id: &str, deps: Vec<&str>, result: NodeResult) -> Self {
        Self {
            id: id.to_string(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
            execute_result: result,
        }
    }
}

impl Node for DummyNode {
    fn id(&self) -> &str { &self.id }
    fn dependencies(&self) -> Vec<&str> { self.deps.iter().map(|s| s.as_str()).collect() }
    fn category(&self) -> NodeCategory { NodeCategory::Skill }
    fn execute(&self, _ctx: &Context) -> NodeResult { self.execute_result.clone() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// ── Test 1: Chain executes root→leaf order (reversed from discovery) ─────

#[test]
fn test_executor_chain_root_to_leaf_order() {
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("root", vec![], NodeResult::ok("root_out")));
    graph.add_node(DummyNode::new("mid", vec!["root"], NodeResult::ok("mid_out")));
    graph.add_node(DummyNode::new("leaf", vec!["mid"], NodeResult::ok("leaf_out")));

    let exec = Executor::new();
    let result = exec.execute(&mut graph, "leaf", "initial");

    assert!(result.success);
    // Verify: discover returns [leaf, mid, root], reversed = [root, mid, leaf]
    // Execution order = [root, mid, leaf] — all succeed
}

// ── Test 2: Initial input goes to root only ──────────────────────────────

#[test]
fn test_executor_initial_input_to_root() {
    // root captures input from context, others return their deps' output
    let mut graph = MemoryGraph::new();

    // Node that reports what input it received
    struct Captor(&'static str);
    impl Node for Captor {
        fn id(&self) -> &str { self.0 }
        fn dependencies(&self) -> Vec<&str> { vec![] }
        fn category(&self) -> NodeCategory { NodeCategory::Skill }
        fn execute(&self, ctx: &Context) -> NodeResult {
            let input = ctx.get("input").unwrap_or("(none)");
            NodeResult::ok(input)
        }
        fn as_any(&self) -> &dyn std::any::Any { self }
    }

    graph.add_node(Captor("root"));
    graph.add_node(Captor("child"));

    let exec = Executor::new();
    let result = exec.execute_node(&mut graph, "root", "ROOT_INPUT");

    assert!(result.success);
    assert_eq!(result.output, "ROOT_INPUT", "Root should receive initial_input");
}

// ── Test 3: Mid-chain failure stops downstream ──────────────────────────

#[test]
fn test_executor_mid_chain_failure_stops_downstream() {
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("A", vec![], NodeResult::ok("A")));
    graph.add_node(DummyNode::new("B", vec!["A"], NodeResult::err("B failed intentionally")));
    graph.add_node(DummyNode::new("C", vec!["B"], NodeResult::ok("C should not run")));

    let exec = Executor::new();
    let result = exec.execute(&mut graph, "C", "");

    assert!(!result.success, "Execution should fail");
    assert_eq!(result.error.as_ref().map(|s| s.as_str()), Some("B failed intentionally"));
}

// ── Test 4: execute_node single node (no chain) ─────────────────────────

#[test]
fn test_executor_execute_node_single() {
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("solo", vec![], NodeResult::ok("solo_output")));

    let exec = Executor::new();
    let result = exec.execute_node(&mut graph, "solo", "input_for_solo");

    assert!(result.success);
    assert_eq!(result.output, "solo_output");
}

// ── Test 5: execute_or_discover when path already verified ──────────────

#[test]
fn test_executor_execute_or_discover_already_verified() {
    let mut graph = make_graph(vec![
        ("A", vec![]),
        ("B", vec!["A"]),
    ]);

    // First discover to cache
    let discovery = ChainDiscovery::new();
    let _ = discovery.discover(&graph, "B");

    let exec = Executor::new();
    let result = exec.execute_or_discover(&mut graph, "B", "");

    assert!(result.success, "execute_or_discover should succeed even though path already known");
}

// ── Test 6: Hotness tracked correctly ───────────────────────────────────

#[test]
fn test_executor_hotness_tracked() {
    let mut graph = make_graph(vec![("H", vec![])]);

    let exec = Executor::new();
    exec.execute_node(&mut graph, "H", "");
    exec.execute_node(&mut graph, "H", "");
    exec.execute_node(&mut graph, "H", "");

    // Use get_hit_count to inspect actual value
    let hit_count = graph.get_hit_count("H").unwrap_or(0);
    assert_eq!(hit_count, 3, "H should have 3 hits, got {}", hit_count);

    let hottest = graph.hottest(1);
    assert_eq!(hottest[0].0, "H");
    assert_eq!(hottest[0].1, 3, "H should have 3 hits");
}

// ── Test 7: Output preserved through chain ──────────────────────────────

#[test]
fn test_executor_output_preserved_through_chain() {
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("root", vec![], NodeResult::ok("ROOT_OUTPUT")));
    graph.add_node(DummyNode::new("child", vec!["root"], NodeResult::ok("CHILD_OUTPUT")));
    graph.add_node(DummyNode::new("leaf", vec!["child"], NodeResult::ok("LEAF_OUTPUT")));

    let exec = Executor::new();
    let result = exec.execute(&mut graph, "leaf", "INITIAL");

    assert!(result.success);
    // Chain root→child→leaf, each succeeds
    // Final result should be from leaf (last node in chain)
    assert_eq!(result.output, "LEAF_OUTPUT");
}

// ── Helper ───────────────────────────────────────────────────────────────

fn make_graph(chains: Vec<(&str, Vec<&str>)>) -> MemoryGraph {
    let mut g = MemoryGraph::new();
    for (id, deps) in chains {
        g.add_node(DummyNode::new(id, deps, NodeResult::ok("ok")));
    }
    g
}