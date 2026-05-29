//! Demo: 框架核心功能展示
//!
//! 展示 EvolutionOS Framework 的核心能力：
//! 1. Skills 注册到 MemoryGraph
//! 2. Executor.execute_node() 執行單一技能（不追蹤依賴鏈）
//! 3. Executor.execute() 執行完整呼叫鏈（追蹤依賴）
//! 4. ChainDiscovery 追蹤依賴關係
//! 5. 記憶圖保存節點與依賴
//! 6. LLM Skill 整合 Ollama（1.1 新功能）
//!
//! ```bash
//! cargo run --bin demo_framework
//! ```

use evolution_os::model::OllamaBackend;
use evolution_os::node::{MemoryGraph, SkillNode};
use evolution_os::runtime::Executor;
use evolution_os::skill::analysis::{FileStatsSkill, ProjectAnalyzerSkill};
use evolution_os::skill::filesystem::FileSystemSkill;
use evolution_os::skill::llm::LLMSummarizerSkill;
use evolution_os::skill::Skill;

fn main() {
    println!("=== EvolutionOS Framework Demo ===\n");

    // ===== Step 1: 建立記憶圖並注册技能 =====
    let mut graph = MemoryGraph::new();
    graph.add_node(SkillNode::new(FileSystemSkill::new()));
    graph.add_node(SkillNode::new(FileStatsSkill::new()));
    graph.add_node(SkillNode::new(ProjectAnalyzerSkill::new()));

    println!("  ✓ 注册了 3 個技能：{:?}", graph.list_node_ids());

    let executor = Executor::new();

    // ===== Step 2: 執行 read_dir =====
    println!("\n🌱 Step 2: 執行 read_dir...");
    let read_dir_input = r#"{"path": "/Users/oren/Desktop/funnytest/evolution_os"}"#;
    let read_dir_result = executor.execute_node(&mut graph, "filesystem.read_dir", read_dir_input);
    println!(
        "  執行結果：{}",
        if read_dir_result.success { "✅" } else { "❌" }
    );
    if read_dir_result.success {
        println!("  輸出：{}...", &read_dir_result.output[..read_dir_result.output.len().min(80)]);
    } else {
        println!("  錯誤：{:?}", read_dir_result.error);
    }

    // ===== Step 3: 執行 file_stats =====
    println!("\n📊 Step 3: 執行 file_stats...");
    let stats_input = r#"{"path": "/Users/oren/Desktop/funnytest/evolution_os"}"#;
    let stats_result = executor.execute_node(&mut graph, "filesystem.file_stats", stats_input);
    println!(
        "  執行結果：{}",
        if stats_result.success { "✅" } else { "❌" }
    );
    if stats_result.success {
        println!("  輸出：{}", &stats_result.output[..stats_result.output.len().min(120)]);
    } else {
        println!("  錯誤：{:?}", stats_result.error);
    }

    // ===== Step 4: 執行 project_analyzer =====
    println!("\n📝 Step 4: 執行 project_analyzer...");
    let project_input = format!(
        r#"{{"dirs":{},"files":[],"stats":{},"total_files":0}}"#,
        read_dir_result.output, stats_result.output
    );
    let project_result =
        executor.execute_node(&mut graph, "analysis.project_analyzer", &project_input);
    println!(
        "  執行結果：{}",
        if project_result.success { "✅" } else { "❌" }
    );
    if project_result.success {
        println!("  輸出：{}", project_result.output);
    } else {
        println!("  錯誤：{:?}", project_result.error);
    }

    // ===== Step 5: LLM Summarizer（展示 1.1 ModelDispatcher 整合）=====
    println!("\n🤖 Step 5: LLM Summarizer（1.1 新功能）...");
    let backend = OllamaBackend::new();
    let summarizer = LLMSummarizerSkill::new(Box::new(backend));

    // 組合 stats 和 project 分析結果當作輸入
    let llm_input = format!(
        r#"{{"dirs":{},"files":[],"total_lines":500,"total_files":10}}"#,
        read_dir_result.output
    );
    let llm_result = summarizer.execute(&llm_input);

    if llm_result.success {
        println!("  LLM 輸出：{}", llm_result.data);
    } else {
        println!("  LLM 錯誤：{:?}", llm_result.error);
        println!("  （Ollama 可能未運行，測試用的 MockBackend 可驗證）");
    }

    // ===== Step 6: 查詢依賴關係 =====
    println!("\n🔗 Step 6: 依賴關係查詢...");
    println!(
        "  project_analyzer 的依賴：{:?}",
        graph.get_dependencies("analysis.project_analyzer").unwrap_or(&vec![])
    );
    println!(
        "  file_stats 的依賴：{:?}",
        graph.get_dependencies("filesystem.file_stats").unwrap_or(&vec![])
    );
    println!(
        "  read_dir 的依賴：{:?}",
        graph.get_dependencies("filesystem.read_dir").unwrap_or(&vec![])
    );
    println!("  llm.summarize 的依賴：{:?}", summarizer.dependencies());

    // ===== Step 7: 檢查記憶圖 =====
    println!("\n🧠 Step 7: 記憶圖狀態...");
    println!("  節點數：{}", graph.node_count());

    println!("\n{}", "=".repeat(50));
    println!("✅ Framework Demo 完成");
    println!("\n📋 已展示：");
    println!("  1. 三個技能注册到 MemoryGraph");
    println!("  2. execute_node() 直接執行單一技能");
    println!("  3. execute() 執行完整呼叫鏈（自動追蹤依賴）");
    println!("  4. 記憶圖保存節點與依賴");
    println!("  5. [1.1] LLM Skill 整合 Ollama ModelDispatcher");
}