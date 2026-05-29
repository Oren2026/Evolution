//! Chain Discovery — 整合測試（邊界條件）
//!
//! 測試 ChainDiscovery BFS 遍歷的邊界條件。

use evolution_os::*;
use evolution_os::node::ChainNode;

struct DummyNode {
    id: String,
    deps: Vec<String>,
}

impl DummyNode {
    fn new(id: &str, deps: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Node for DummyNode {
    fn id(&self) -> &str { &self.id }
    fn dependencies(&self) -> Vec<&str> { self.deps.iter().map(|s| s.as_str()).collect() }
    fn category(&self) -> NodeCategory { NodeCategory::Skill }
    fn execute(&self, _ctx: &Context) -> NodeResult { NodeResult::ok("ok") }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn make_graph(chains: Vec<(&str, Vec<&str>)>) -> MemoryGraph {
    let mut g = MemoryGraph::new();
    for (id, deps) in chains {
        g.add_node(DummyNode::new(id, deps));
    }
    g
}

// ── Test 1: Long chain (65 nodes) ───────────────────────────────────────

#[test]
fn test_chain_discovery_long_chain_max_depth() {
    let mut graph = MemoryGraph::new();
    for i in 0..65 {
        let deps = if i > 0 { vec![format!("n{}", i - 1)] } else { vec![] };
        let deps_refs: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
        graph.add_node(DummyNode::new(&format!("n{}", i), deps_refs));
    }

    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph, "n64").expect("n64 at depth 64 should be discoverable");
    assert_eq!(result.path.len(), 65);
}

// ── Test 2: Diamond deps (A→B, A→C, B→D, C→D) ───────────────────────────

#[test]
fn test_chain_discovery_diamond_deps() {
    let graph = make_graph(vec![
        ("A", vec![]),
        ("B", vec!["A"]),
        ("C", vec!["A"]),
        ("D", vec!["B", "C"]),
    ]);

    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph, "D").expect("D should be discoverable");

    assert_eq!(result.path[0], "D");
    assert!(result.path.contains(&"B".to_string()), "B should be in path: {:?}", result.path);
    assert!(result.path.contains(&"C".to_string()), "C should be in path: {:?}", result.path);
    assert!(result.path.contains(&"A".to_string()), "A should be in path: {:?}", result.path);
}

// ── Test 3: Disconnected trees ───────────────────────────────────────────

#[test]
fn test_chain_discovery_disconnected_trees() {
    // Build: A(no deps)←B  and  C(no deps)←D
    // discover("B") → path [B, A] (follows B→A)
    // discover("D") → path [D, C] (follows D→C)
    let graph = make_graph(vec![
        ("A", vec![]),
        ("B", vec!["A"]),
        ("C", vec![]),
        ("D", vec!["C"]),
    ]);

    let discovery = ChainDiscovery::new();

    let result = discovery.discover(&graph, "D").expect("D should be discoverable");
    assert_eq!(result.path, vec!["D", "C"]);

    let result_b = discovery.discover(&graph, "B").expect("B should be discoverable");
    assert_eq!(result_b.path, vec!["B", "A"]);
}

// ── Test 4: Orphan root (no deps) ───────────────────────────────────────

#[test]
fn test_chain_discovery_orphan_root() {
    let graph = make_graph(vec![("lonely", vec![])]);

    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph, "lonely").expect("lonely should be discoverable");

    assert_eq!(result.leaf_id, "lonely");
    assert_eq!(result.path, vec!["lonely"]);
    assert_eq!(result.depth, 0);
}

// ── Test 5: Verified fast path ──────────────────────────────────────────

#[test]
fn test_chain_discovery_verified_fast_path() {
    let mut graph = make_graph(vec![
        ("X", vec![]),
        ("Y", vec!["X"]),
        ("Z", vec!["Y"]),
    ]);

    let mut chain = ChainNode::new("Z", vec!["Z".to_string(), "Y".to_string(), "X".to_string()]);
    chain.mark_verified();
    graph.register_chain(chain);

    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph, "Z").expect("Z should be discoverable");

    assert!(result.verified);
    assert_eq!(result.path, vec!["Z", "Y", "X"]);
}

// ── Test 6: verify_and_register creates verified chain ───────────────────

#[test]
fn test_chain_discovery_verify_and_register() {
    let mut graph = make_graph(vec![
        ("P", vec![]),
        ("Q", vec!["P"]),
        ("R", vec!["Q"]),
    ]);

    let discovery = ChainDiscovery::new();
    let result = discovery.verify_and_register(&mut graph, "R").expect("R should be discoverable");

    assert!(result.verified);
    assert_eq!(graph.chain_count(), 1);

    let result2 = discovery.discover(&graph, "R").expect("R should still be discoverable");
    assert!(result2.verified);
}

// ── Test 7: No infinite loop on visited nodes ───────────────────────────

#[test]
fn test_chain_discovery_no_infinite_loop() {
    // Two separate linear chains — no cycles, but visited set prevents re-traversal
    let mut graph3 = MemoryGraph::new();
    graph3.add_node(DummyNode::new("L1", vec![]));
    graph3.add_node(DummyNode::new("L2", vec!["L1"]));

    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph3, "L2").expect("L2 should be discoverable");
    assert_eq!(result.path, vec!["L2", "L1"]);
}