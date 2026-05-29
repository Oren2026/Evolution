//! analyze — 一鍵 Planner → Executor

use crate::commands::new_project::evolution_root;
use std::path::PathBuf;

/// 分析並執行任務
pub fn analyze(task: &str, project_name: Option<&str>) {
    // 決定專案
    let project_path = match project_name {
        Some(name) => {
            let p = evolution_root().join("projects").join(name);
            if !p.exists() {
                eprintln!("✗ 專案 '{}' 不存在，請先建立：evolution new {}", name, name);
                std::process::exit(1);
            }
            p
        }
        None => {
            // 自動建立暫時專案
            let name = format!("anon_{}", chrono::Utc::now().timestamp());
            let p = evolution_root().join("projects").join(&name);
            eprintln!("(未指定專案，建立暫時專案: {})", name);
            crate::commands::new_project::new_project(&name);
            p
        }
    };

    println!();
    println!("═══ Evolution OS Analyze ═══");
    println!("任務: {}", task);
    println!("專案: {}", project_path.display());
    println!();

    // 執行 KernelRuntime（Planner → Executor）
    let manifest = {
        use evolution_os::kernel::kernel_runtime::KernelRuntime;

        let mut kr = KernelRuntime::new();
        kr.boot();

        // Spawn planner + executor
        let _ppid = kr.spawn_planner();
        let _epid = kr.spawn_executor();

        // 執行 planner（sync）
        let manifest = kr.run_planner_sync(task);
        println!("✓ Planner 完成");
        println!("  stage: {:?}", manifest.stage);
        println!("  work_mode: {:?}", manifest.work_mode);
        println!("  complexity: {:?}", manifest.complexity);
        println!();

        manifest
    };

    // 執行 executor（sync）
    let output = {
        use evolution_os::kernel::kernel_runtime::KernelRuntime;

        let mut kr = KernelRuntime::new();
        kr.boot();
        let _ppid = kr.spawn_planner();
        let _epid = kr.spawn_executor();

        let output = kr.run_executor_sync(&manifest);
        println!("✓ Executor 完成");
        println!("  output: {}", output);
        println!();

        output
    };

    // 寫入 manifest.ev
    let manifest_path = project_path.join("manifest.ev");
    if let Ok(json) = manifest.to_json() {
        let ev_content = format!(
            "# Evolution OS Manifest\n# 任務: {}\n# 時間: {}\n\n{}\n",
            task,
            chrono::Utc::now().to_rfc3339(),
            json
        );
        if let Err(e) = std::fs::write(&manifest_path, ev_content) {
            eprintln!("✗ 無法寫入藍圖: {}", e);
        } else {
            println!("✓ 藍圖已寫入: {}", manifest_path.display());
        }
    }

    // 更新 evolution.yaml
    let config_path = project_path.join("evolution.yaml");
    let config_text = format!(
        r#"# Evolution OS 專案設定
name: {}
created: "{}"
task: "{}"
description: ""

skills: []

planner:
  work_mode: {:?}
  complexity_threshold: {}

executor:
  max_tier: 10
"#,
        project_path.file_name().unwrap_or_default().to_string_lossy(),
        chrono::Utc::now().to_rfc3339(),
        task,
        manifest.work_mode,
        manifest.complexity.context_complexity
    );
    if let Err(e) = std::fs::write(&config_path, config_text) {
        eprintln!("✗ 無法更新設定檔: {}", e);
    }

    println!();
    println!("═══ 完成 ═══");
    println!("藍圖: {}", manifest_path.display());
    println!("設定: {}", config_path.display());
}