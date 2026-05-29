//! GraphExecutor — 整合測試
//!
//! 測試 GraphExecutor 的 tier-based 執行、並行、上下文傳遞。

use evolution_os::*;
use evolution_os::compiler::ExecutionGraph;
use evolution_os::planner::manifest::{EstimatedNode, Manifest};
use evolution_os::planner::stages::Stage;
use std::any::Any;
use std::sync::{Arc, Mutex};

struct DummyNode {
    id: String,
    deps: Vec<String>,
    result: NodeResult,
}

impl DummyNode {
    fn new(id: &str, deps: Vec<&str>, result: NodeResult) -> Self {
        Self {
            id: id.to_string(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
            result,
        }
    }
}

impl Node for DummyNode {
    fn id(&self) -> &str { &self.id }
    fn dependencies(&self) -> Vec<&str> { self.deps.iter().map(|s| s.as_str()).collect() }
    fn category(&self) -> NodeCategory { NodeCategory::Skill }
    fn execute(&self, _ctx: &Context) -> NodeResult { self.result.clone() }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn make_exec_graph(nodes: Vec<(&str, Vec<&str>)>) -> ExecutionGraph {
    let estimated_nodes: Vec<EstimatedNode> = nodes
        .iter()
        .map(|(id, deps)| EstimatedNode {
            id: (*id).to_string(),
            role: "r".to_string(),
            handles: vec![],
            depends_on: deps.iter().map(|d| (*d).to_string()).collect(),
        })
        .collect();

    let manifest = Manifest {
        version: "0.1.0".to_string(),
        task: "test".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        stage: Stage::Complete,
        requirements: vec![],
        questions: vec![],
        converged: true,
        complexity: Default::default(),
        estimated_nodes,
        work_mode: Default::default(),
        dispatch: Default::default(),
        optimized_prompt: Default::default(),
    };

    ExecutionGraph::from_manifest(&manifest).unwrap()
}

// ── Test 1: Simple sequential execution ─────────────────────────────────

#[test]
fn test_graph_executor_sequential_tiers() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("root", vec![], NodeResult::ok("root_out")));
    graph.add_node(DummyNode::new("child", vec!["root"], NodeResult::ok("child_out")));

    let exec_graph = make_exec_graph(vec![
        ("root", vec![]),
        ("child", vec!["root"]),
    ]);

    let results = exec.execute(&mut graph, &exec_graph);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.success));
}

// ── Test 2: Failed node does not block tier ───────────────────────────────

#[test]
fn test_graph_executor_failed_node_propagates() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("ok", vec![], NodeResult::ok("ok_out")));
    graph.add_node(DummyNode::new("fail", vec![], NodeResult::err("intentional failure")));

    let exec_graph = make_exec_graph(vec![("ok", vec![]), ("fail", vec![])]);

    let results = exec.execute(&mut graph, &exec_graph);
    assert_eq!(results.len(), 2);

    let fail_result = results.iter().find(|r| !r.success).unwrap();
    assert!(!fail_result.success);
    assert_eq!(fail_result.error.as_ref().map(|s| s.as_str()), Some("intentional failure"));
}

// ── Test 3: execute_node single node ─────────────────────────────────────

#[test]
fn test_graph_executor_execute_node_single() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("solo", vec![], NodeResult::ok("solo_out")));

    let result = exec.execute_node(&mut graph, "solo", "my_input");
    assert!(result.success);
    assert_eq!(result.output, "solo_out");
}

// ── Test 4: execute_node none-existent node ───────────────────────────────

#[test]
fn test_graph_executor_execute_node_missing() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();

    let result = exec.execute_node(&mut graph, "ghost", "");
    assert!(!result.success);
    assert!(result.error.is_some());
}

// ── Test 5: Empty execution graph ─────────────────────────────────────────

#[test]
fn test_graph_executor_empty_graph() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("unused", vec![], NodeResult::ok("out")));

    let exec_graph = make_exec_graph(vec![]);
    let results = exec.execute(&mut graph, &exec_graph);
    assert!(results.is_empty());
}

// ── Test 6: hit_count incremented on execute ─────────────────────────────

#[test]
fn test_graph_executor_hit_tracked() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("X", vec![], NodeResult::ok("X")));

    let exec_graph = make_exec_graph(vec![("X", vec![])]);
    exec.execute(&mut graph, &exec_graph);

    // GraphExecutor calls graph.hit() for each executed node
    let hit_count = graph.get_hit_count("X").unwrap_or(0);
    assert_eq!(hit_count, 1, "X should have 1 hit after one execution");
}

// ── Test 7: Multiple tiers execute in order ────────────────────────────────

#[test]
fn test_graph_executor_three_tiers() {
    let exec = GraphExecutor::new();
    let mut graph = MemoryGraph::new();
    graph.add_node(DummyNode::new("A", vec![], NodeResult::ok("A")));
    graph.add_node(DummyNode::new("B", vec!["A"], NodeResult::ok("B")));
    graph.add_node(DummyNode::new("C", vec!["B"], NodeResult::ok("C")));

    let exec_graph = make_exec_graph(vec![
        ("A", vec![]),
        ("B", vec!["A"]),
        ("C", vec!["B"]),
    ]);

    let results = exec.execute(&mut graph, &exec_graph);
    assert_eq!(results.len(), 3);
    // All succeed
    assert!(results.iter().all(|r| r.success));
}