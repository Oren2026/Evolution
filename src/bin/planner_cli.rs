//! Planner CLI — 任務規劃與分工決策命令行工具
//!
//! 用法：
//!   cargo run --bin planner -- "幫我建一個庫存管理系統"
//!   cargo run --bin planner -- --interactive
//!   cargo run --bin planner -- --llm -- "幫我建一個庫存管理系統"
//!
//! 輸出：JSON Manifest

use evolution_os::planner::Manifest;
#[cfg(feature = "llm")]
use evolution_os::model::OllamaBackend;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    // --check 模式：只做系統檢查不安裝
    if args.get(1).map(|s| s == "--check").unwrap_or(false) {
        run_system_check();
        return;
    }

    // --install 模式：檢查 + 自動安裝
    if args.get(1).map(|s| s == "--install").unwrap_or(false) {
        run_system_check();
        println!();
        run_auto_install();
        return;
    }

    // Pre-flight check（無 --check flag 也做快速檢查，缺少時警告）
    preflight_check();

    // --llm 模式
    let use_llm = args.get(1).map(|s| s == "--llm").unwrap_or(false);
    // 任務：跳過 --llm(1)，若 args[2] 是 "--" 则从 3 开始，否则从 2 开始
    let task_start = if use_llm {
        if args.get(2).map(|s| s.as_str()) == Some("--") {
            3
        } else {
            2
        }
    } else {
        1
    };

    // --interactive 模式
    if args.get(task_start).map(|s| s == "--interactive" || s == "-i").unwrap_or(false) {
        println!("📋 Evolution Planner — 互動模式");
        println!("輸入你的任務描述（空白行結束）：");
        let mut task = String::new();
        loop {
            let mut line = String::new();
            use std::io::Read;
            match std::io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if line.trim().is_empty() {
                        break;
                    }
                    task.push_str(&line);
                }
                Err(_) => break,
            }
        }
        if task.trim().is_empty() {
            eprintln!("任務描述為空");
            std::process::exit(1);
        }
        run_planner(&task, use_llm);
        return;
    }

    // 直接模式：剩下的 args 組合成任務描述
    let task = args[task_start..].join(" ");
    if task.trim().is_empty() {
        eprintln!("任務描述為空");
        std::process::exit(1);
    }
    run_planner(&task, use_llm);
}

fn run_planner(task: &str, use_llm: bool) {
    println!("\n🔍 分析任務：{}\n", task);
    println!("{}", "=".repeat(60));

    #[cfg(feature = "llm")]
    let manifest = if use_llm {
        use evolution_os::model::OllamaBackend;
        let backend = OllamaBackend::new();
        println!("🤖 LLM 模式：使用 llama3 分析\n");
        Manifest::from_task_with_llm(task, &backend)
    } else {
        Manifest::from_task(task)
    };

    #[cfg(not(feature = "llm"))]
    let manifest = {
        if use_llm {
            println!("⚠️  尚未編譯 LLM 功能，使用規則模式。");
            println!("   重新編譯：cargo build --features llm\n");
        }
        Manifest::from_task(task)
    };

    // 輸出摘要
    println!("\n📊 複雜度指標");
    println!("  推理分支數：{}", manifest.complexity.reasoning_branches);
    println!("  領域多樣性：{}", manifest.complexity.domain_diversity);
    println!("  語境複雜度：{:.2}", manifest.complexity.context_complexity);

    println!("\n🔀 分工模式");
    let mode_str = match manifest.work_mode {
        evolution_os::planner::WorkMode::Solo => "Solo（單一節點）",
        evolution_os::planner::WorkMode::Fork => "Fork（多節點分工）",
    };
    println!("  模式：{}", mode_str);
    println!("  理由：{}", manifest.dispatch.rationale);
    println!("  預估節點數：{}", manifest.dispatch.estimated_nodes);

    if !manifest.dispatch.domain_tags.is_empty() {
        println!("  領域標籤：{:?}", manifest.dispatch.domain_tags);
    }

    println!("\n📋 需求項目（{} 項）", manifest.requirements.len());
    for req in &manifest.requirements {
        println!("  [{}] {} ({})", req.priority, req.requirement, req.domain);
    }

    if !manifest.questions.is_empty() {
        println!("\n❓ 待確認問題（{} 項）", manifest.questions.len());
        for q in &manifest.questions {
            println!("  {}: {} ({})", q.id, q.question, q.category);
            println!("       影響：{}", q.impact);
        }
    } else {
        println!("\n✅ 所有需求已確認，無待決問題");
    }

    if manifest.work_mode == evolution_os::planner::WorkMode::Fork {
        println!("\n👥 預估節點結構（{} 個）", manifest.estimated_nodes.len());
        for node in &manifest.estimated_nodes {
            print!("  {}（{}）", node.id, node.role);
            if node.depends_on.is_empty() {
                println!(" - 無依賴");
            } else {
                println!(" → 依賴 {:?}", node.depends_on);
            }
            println!("    處理：{:?}", node.handles);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("\n📄 完整 JSON Manifest：\n");

    match manifest.to_json() {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("JSON 輸出失敗：{}", e),
    }
}

// ===== 系統檢查與安裝 =====

fn print_help() {
    eprintln!("Evolution Planner CLI");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  planner <任務描述>                      分析任務（規則模式）");
    eprintln!("  planner --llm -- <任務描述>             分析任務（LLM 模式）");
    eprintln!("  planner --check                         檢查系統環境");
    eprintln!("  planner --install                      檢查並自動安裝缺少的元件");
    eprintln!("  planner --interactive                   互動模式");
    eprintln!();
    eprintln!("範例:");
    eprintln!("  planner --llm -- 幫我建一個庫存管理系統");
    eprintln!("  planner --check");
    eprintln!("  planner --install");
}

fn run_system_check() {
    println!("🔍 系統環境檢查\n");
    for report in evolution_os::system::SystemReport::all() {
        report.print_summary();
    }
}

fn preflight_check() {
    let missing: Vec<(String, bool)> = evolution_os::system::Installer::quick_check();
    let any_missing = missing.iter().any(|(_, present)| !present);

    if any_missing {
        println!("⚠️  系統檢查：發現缺少元件");
        for (name, present) in &missing {
            if !present {
                println!("  ❌ {}", name);
            }
        }
        println!();
        println!("執行 `planner --install` 自動安裝，或手動安裝後再試。");
        println!();
    }
}

fn run_auto_install() {
    println!("🚀 開始自動安裝...\n");
    let installer = evolution_os::system::Installer::new();
    let ok = installer.install_missing();
    println!();
    if ok {
        println!("✅ 所有元件已就緒，任務規劃系統已準備好！");
    } else {
        println!("⚠️  部分元件安裝失敗，請參考上方錯誤訊息手動處理。");
    }
}