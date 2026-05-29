//! Demo: 目錄分析器
//!
//! 展示 EvolutionOS 的完整執行鏈。
//!
//! ```bash
//! cargo run --bin analyze_project
//! ```

use evolution_os::skill::analysis::{FileStatsSkill, ProjectAnalyzerSkill};
use evolution_os::skill::filesystem::FileSystemSkill;
use evolution_os::skill::Skill;
use evolution_os::{Context, Node};

fn main() {
    println!("=== EvolutionOS Demo: 專案分析 ===\n");

    // 目標路徑
    let target_path = "/Users/oren/Desktop/funnytest/evolution_os";

    println!("分析目標：{}\n", target_path);

    // ===== Step 1: 讀取目錄結構 =====
    println!("📁 Step 1: 讀取目錄結構...");
    let fs_skill = FileSystemSkill::new();
    let fs_node = evolution_os::SkillNode::new(fs_skill);

    let fs_input = serde_json::json!({ "path": target_path }).to_string();
    let mut ctx1 = Context::new("filesystem.read_dir");
    ctx1.insert("input", &fs_input);
    let result1 = fs_node.execute(&ctx1);

    if !result1.success {
        println!("  ✗ 讀取失敗: {:?}", result1.error);
        return;
    }

    let dir_data: serde_json::Value = serde_json::from_str(&result1.output).unwrap();
    let dirs = dir_data["dirs"].as_array().map(|a| a.len()).unwrap_or(0);
    let files = dir_data["files"].as_array().map(|a| a.len()).unwrap_or(0);
    println!("  ✓ 找到 {} 個目錄, {} 個檔案", dirs, files);

    // ===== Step 2: 分析檔案統計 =====
    println!("\n📊 Step 2: 分析檔案統計...");
    let stats_skill = FileStatsSkill::new();
    let stats_node = evolution_os::SkillNode::new(stats_skill);

    // 只取檔案（不取目錄），避免浪費時間統計目錄
    let all_entries: Vec<serde_json::Value> = dir_data["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let name_val = e["name"].as_str().unwrap_or("");
            let full_path = format!("{}/{}", target_path, name_val);
            serde_json::json!({
                "path": full_path,
                "name": name_val
            })
        })
        .collect();

    let stats_input = serde_json::json!({ "files": all_entries }).to_string();
    let mut ctx2 = Context::new("filesystem.file_stats");
    ctx2.insert("input", &stats_input);
    let result2 = stats_node.execute(&ctx2);

    if !result2.success {
        println!("  ✗ 統計失敗: {:?}", result2.error);
        return;
    }

    let stats_data: serde_json::Value = serde_json::from_str(&result2.output).unwrap();
    let total_files = stats_data["total_files"].as_u64().unwrap_or(0);
    let total_lines: u64 = stats_data["stats"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s.get("lines").and_then(|l| l.as_u64()))
        .sum();
    println!(
        "  ✓ 分析了 {} 個檔案, 總行數: {}",
        total_files, total_lines
    );

    // ===== Step 3: 產生專案摘要 =====
    println!("\n📝 Step 3: 產生專案摘要...");
    let analyzer_skill = ProjectAnalyzerSkill::new();
    let analyzer_node = evolution_os::SkillNode::new(analyzer_skill);

    let summary_input = serde_json::json!({
        "dirs": dir_data["dirs"],
        "files": dir_data["files"],
        "stats": stats_data
    })
    .to_string();

    let mut ctx3 = Context::new("analysis.project_analyzer");
    ctx3.insert("input", &summary_input);
    let result3 = analyzer_node.execute(&ctx3);

    if !result3.success {
        println!("  ✗ 摘要失敗: {:?}", result3.error);
        return;
    }

    let summary: serde_json::Value = serde_json::from_str(&result3.output).unwrap();

    // ===== 輸出結果 =====
    println!("\n{}", "=".repeat(50));
    println!("✅ 專案分析完成！");
    println!("{}", "=".repeat(50));
    println!();
    println!("📋 摘要：{}", summary["summary"]);
    println!("📁 模組：{:?}", summary["modules"]);
    println!("📄 總檔案：{}", summary["total_files"]);
    println!("📝 總行數：{}", summary["total_lines"]);

    println!(
        "\n⛓️ 執行鏈：analyze_project → read_directory → analyze_file_stats → summarize_project"
    );
    println!("\n✅ Demo 完成");
}