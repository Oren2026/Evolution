//! Evolution Agent — 背景任務執行器
//!
//! 用法：
//!   cargo run --bin evolution_agent -- "幫我建庫存系統"
//!   .\evolution.exe agent "幫我建庫存系統"
//!
//! 流程：
//!   1. Planner 分析任務複雜度 → Manifest
//!   2. 背景起 OpenCode worker
//!   3. OpenCode 實作，完成後 auto commit + tag + push

use std::env;
use std::process::Command;
use std::thread;
use std::time::Duration;

use evolution_os::planner::Manifest;
use evolution_os::planner::WorkMode;

const AGENT_BANNER: &str = r#"
╔══════════════════════════════════════════════╗
║         Evolution Agent — v0.1.0            ║
║   背景任務執行器 · OpenCode 協作 · Auto-push ║
╚══════════════════════════════════════════════╝
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{}", AGENT_BANNER);

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    let task = args[1..].join(" ");
    if task.trim().is_empty() {
        eprintln!("❌ 任務描述為空");
        std::process::exit(1);
    }

    // Step 1: Planner 分析
    println!("\n📋 Step 1：Planner 分析任務...\n");
    let manifest = Manifest::from_task(&task);

    println!("  推理分支數：{}", manifest.complexity.reasoning_branches);
    println!("  領域多樣性：{}", manifest.complexity.domain_diversity);
    println!("  語境複雜度：{:.2}", manifest.complexity.context_complexity);
    println!("  分工模式：{}",
        match manifest.work_mode {
            WorkMode::Solo => "Solo（單一節點）",
            WorkMode::Fork => "Fork（多節點分工）",
        });
    println!("  理由：{}", manifest.dispatch.rationale);

    // Step 2: 背景起 OpenCode
    println!("\n🚀 Step 2：啟動 OpenCode worker...\n");

    let opencode_prompt = build_opencode_prompt(&task, &manifest);

    println!("  任務描述：{}", task);
    println!("  預估節點數：{}", manifest.dispatch.estimated_nodes);
    if !manifest.dispatch.domain_tags.is_empty() {
        println!("  領域標籤：{:?}", manifest.dispatch.domain_tags);
    }

    // 檢查 OpenCode 是否可用
    let opencode_check = Command::new("opencode")
        .arg("--version")
        .output();

    match opencode_check {
        Ok(output) if output.status.success() => {
            println!("\n  ✅ OpenCode 已就緒");
            println!("  {}", String::from_utf8_lossy(&output.stdout).trim());
        }
        _ => {
            eprintln!("\n  ⚠️  OpenCode 未安裝或未認證");
            eprintln!("  請先執行：npm i -g opencode-ai@latest");
            eprintln!("  然後執行：opencode auth login");
            eprintln!("\n  或在 Windows PowerShell 中：npm install -g opencode-ai");
        }
    }

    println!("\n📦 啟動背景 worker...\n");

    // 在背景啟動 OpenCode
    // 我們用 opencode serve 模式，然後用 opencode run 執行任務
    let workdir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    // 用 spawn 背景執行 opencode run
    let child = Command::new("opencode")
        .args(["run", &opencode_prompt])
        .current_dir(&workdir)
        .spawn();

    match child {
        Ok(mut child) => {
            println!("  ✅ OpenCode worker 已啟動 (PID: {})", child.id());
            println!("  工作目錄：{}", workdir);
            println!("  任務：{}", task);

            // 等待一下讓 OpenCode 開始
            thread::sleep(Duration::from_secs(2));

            // 等待 child 完成，期間可以檢查
            println!("\n⏳ 等待 OpenCode 完成...\n");

            let result = child.wait();

            match result {
                Ok(exit_status) => {
                    if exit_status.success() {
                        println!("\n✅ OpenCode worker 完成");

                        // Step 3: Auto commit + tag + push
                        auto_git_workflow();
                    } else {
                        println!("\n⚠️  OpenCode worker 結束，退出碼：{}", exit_status);
                        println!("  請手動檢查 git status");
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ 等待 OpenCode 時發生錯誤：{}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ 無法啟動 OpenCode：{}", e);
            eprintln!("\n請確認：");
            eprintln!("  1. OpenCode 已安裝：npm i -g opencode-ai@latest");
            eprintln!("  2. OpenCode 已認證：opencode auth login");
            eprintln!("  3. 在正確的目錄執行");
            std::process::exit(1);
        }
    }
}

fn build_opencode_prompt(task: &str, manifest: &Manifest) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "你是 Evolution OS 的實作幫手。任務：{}\n\n",
        task
    ));

    prompt.push_str("任務複雜度分析：\n");
    prompt.push_str(&format!("  - 推理分支數：{}\n", manifest.complexity.reasoning_branches));
    prompt.push_str(&format!("  - 領域多樣性：{}\n", manifest.complexity.domain_diversity));
    prompt.push_str(&format!("  - 語境複雜度：{:.2}\n", manifest.complexity.context_complexity));
    prompt.push_str(&format!("  - 分工模式：{}\n",
        match manifest.work_mode {
            WorkMode::Solo => "Solo（單一節點）",
            WorkMode::Fork => "Fork（多節點分工）",
        }));
    prompt.push_str(&format!("  - 預估節點數：{}\n", manifest.dispatch.estimated_nodes));

    if !manifest.dispatch.domain_tags.is_empty() {
        prompt.push_str(&format!("  - 領域標籤：{:?}\n", manifest.dispatch.domain_tags));
    }

    if !manifest.requirements.is_empty() {
        prompt.push_str("\n需求項目：\n");
        for (i, req) in manifest.requirements.iter().enumerate() {
            prompt.push_str(&format!("  {}. {} ({})\n", i + 1, req.requirement, req.domain));
        }
    }

    if !manifest.questions.is_empty() {
        prompt.push_str("\n待確認問題：\n");
        for q in &manifest.questions {
            prompt.push_str(&format!("  - {}: {} ({})\n", q.id, q.question, q.category));
        }
    }

    prompt.push_str(
        "\n請開始實作。完成後：\n\
        1. 確保程式碼可以編譯（cargo build 或對應平台的 build 指令）\n\
        2. 執行相關測試確認功能正常\n\
        3. Commit 你的變更，格式：[Evolution] 完成任務：{}\n\
        4. Push 到 origin\n\
        5. 創建 tag，格式：v0.x.x（例如 v0.1.0）\n\
        如果是複雜任務，自動分工給多個 worker 處理不同的領域。\n"
    );

    prompt
}

fn auto_git_workflow() {
    println!("\n📦 Step 3：Git Auto-push Workflow...\n");

    // 檢查 git status
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    match status {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.trim().is_empty() {
                println!("  ✅ 沒有待提交的變更");
            } else {
                println!("  📝 待提交變更：");
                for line in output_str.lines().take(10) {
                    println!("    {}", line);
                }

                // Stage 所有變更
                let add = Command::new("git")
                    .args(["add", "-A"])
                    .output();

                match add {
                    Ok(_) => {
                        // 嘗試自動 commit（如果沒有的話）
                        let commit_msg = format!(
                            "[Evolution Agent] 任務完成 @ {}",
                            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
                        );

                        let commit = Command::new("git")
                            .args(["commit", "-m", &commit_msg])
                            .output();

                        match commit {
                            Ok(c) if c.status.success() => {
                                println!("  ✅ 自動 commit 成功");
                            }
                            _ => {
                                // 可能沒東西要 commit，繼續
                                println!("  ℹ️  沒有變更需要 commit");
                            }
                        }

                        // Push
                        let push = Command::new("git")
                            .args(["push", "origin", "HEAD"])
                            .output();

                        match push {
                            Ok(p) if p.status.success() => {
                                println!("  ✅ Push 成功");

                                // 嘗試打 tag
                                let tag_name = format!(
                                    "v0.{}.0",
                                    chrono::Utc::now().timestamp() % 1000
                                );
                                let tag = Command::new("git")
                                    .args(["tag", "-a", &tag_name, "-m", &commit_msg])
                                    .output();

                                match tag {
                                    Ok(t) if t.status.success() => {
                                        println!("  🏷️  Tag {} 已建立", tag_name);

                                        // Push tags
                                        let push_tag = Command::new("git")
                                            .args(["push", "origin", "--tags"])
                                            .output();

                                        match push_tag {
                                            Ok(pt) if pt.status.success() => {
                                                println!("  ✅ Tags pushed");
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {
                                println!("  ⚠️  Push 失敗，請手動檢查");
                            }
                        }
                    }
                    Err(_) => {
                        println!("  ⚠️  無法 stage 變更");
                    }
                }
            }
        }
        Err(e) => {
            println!("  ⚠️  無法讀取 git status：{}", e);
        }
    }

    println!("\n{}", "═".repeat(50));
    println!("✅ Evolution Agent 完成");
    println!("\n💡 查看進度：git log --oneline");
    println!("💡 查看標籤：git tag");
    println!("💡 查看差異：git diff origin/main");
}

fn print_help() {
    eprintln!("Evolution Agent — 背景任務執行器");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  evolution_agent <任務描述>    啟動背景 OpenCode worker");
    eprintln!();
    eprintln!("範例:");
    eprintln!("  evolution_agent \"幫我建一個部落格系統\"");
    eprintln!("  evolution_agent \"實作用戶登入功能\"");
    eprintln!();
    eprintln!("前置需求:");
    eprintln!("  1. npm i -g opencode-ai@latest");
    eprintln!("  2. opencode auth login");
}