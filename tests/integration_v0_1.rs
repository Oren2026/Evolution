//! Integration Tests — v0.1
//!
//! 測試 Node、MemoryGraph、ChainDiscovery 的整合運作。

use evolution_os::*;

fn dummy_skill_node(id: &str, deps: Vec<&str>) -> impl Node {
    struct D(String, Vec<String>);
    impl Node for D {
        fn id(&self) -> &str { &self.0 }
        fn dependencies(&self) -> Vec<&str> { self.1.iter().map(|s| s.as_str()).collect() }
        fn category(&self) -> NodeCategory { NodeCategory::Skill }
        fn execute(&self, _ctx: &Context) -> NodeResult { NodeResult::ok("ok") }
        fn as_any(&self) -> &dyn std::any::Any { self }
    }
    D(id.to_string(), deps.iter().map(|s| s.to_string()).collect())
}

#[test]
fn test_integration_node_in_graph_and_chain_discovery() {
    let mut graph = MemoryGraph::new();

    // 建立依賴鏈：A → B → C（leaf=C）
    graph.add_node(dummy_skill_node("A", vec![]));
    graph.add_node(dummy_skill_node("B", vec!["A"]));
    graph.add_node(dummy_skill_node("C", vec!["B"]));

    // 探索呼叫鏈
    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph, "C").unwrap();

    assert_eq!(result.path, vec!["C", "B", "A"]);
    assert_eq!(result.depth, 2);
    assert!(!result.verified); // 尚未驗證
}

#[test]
fn test_integration_verify_and_register_chain() {
    let mut graph = MemoryGraph::new();
    graph.add_node(dummy_skill_node("X", vec![]));
    graph.add_node(dummy_skill_node("Y", vec!["X"]));
    graph.add_node(dummy_skill_node("Z", vec!["Y"]));

    let discovery = ChainDiscovery::new();
    let result = discovery.verify_and_register(&mut graph, "Z").unwrap();

    assert!(result.verified);
    assert_eq!(graph.chain_count(), 1);

    // 再次探索應該直接拿到已驗證的鏈
    let result2 = discovery.discover(&graph, "Z").unwrap();
    assert!(result2.verified);
}

#[test]
fn test_integration_registry_and_memory_graph() {
    let mut graph = MemoryGraph::new();

    graph.add_node(dummy_skill_node("skill_html", vec![]));
    graph.add_node(dummy_skill_node("skill_css", vec![]));
    graph.add_node(dummy_skill_node("skill_js", vec!["skill_html"]));

    // 確認節點存在
    assert!(graph.has_node("skill_html"));
    assert!(graph.has_node("skill_css"));
    assert!(graph.has_node("skill_js"));

    // 確認依賴關係
    let deps = graph.get_dependencies("skill_js").unwrap();
    assert_eq!(deps, &vec!["skill_html"]);

    // 熱度追蹤
    graph.hit("skill_html");
    graph.hit("skill_html");
    graph.hit("skill_css");

    let hottest = graph.hottest(3);
    assert_eq!(hottest[0].0, "skill_html");
    assert_eq!(hottest[0].1, 2);
}

#[test]
fn test_integration_cycle_detection() {
    let mut graph = MemoryGraph::new();

    // 模擬一個簡單的循環：A → B（不是真正的循環，只是測試 visited 追蹤）
    graph.add_node(dummy_skill_node("P", vec![]));
    graph.add_node(dummy_skill_node("Q", vec!["P"]));
    graph.add_node(dummy_skill_node("R", vec!["Q"]));

    let discovery = ChainDiscovery::new();
    let result = discovery.discover(&graph, "R").unwrap();

    // R → Q → P，沒有循環
    assert_eq!(result.path, vec!["R", "Q", "P"]);
}

#[test]
fn test_integration_context_tracking() {
    let mut ctx = Context::new("leaf_node");
    ctx.push_parent("parent_1");
    ctx.push_parent("parent_2");
    ctx.insert("key", "value");

    assert_eq!(ctx.leaf_id, "leaf_node");
    assert_eq!(ctx.get_parents(), &["parent_1", "parent_2"]);
    assert_eq!(ctx.get("key"), Some("value"));
}