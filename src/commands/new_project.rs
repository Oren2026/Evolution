//! new_project — 建立新專案

use std::fs;
use std::path::PathBuf;

/// Evolution OS 根目錄（pub for commands）
pub fn evolution_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".evolution")
}

fn project_path(name: &str) -> PathBuf {
    evolution_root().join("projects").join(name)
}

pub fn new_project(name: &str) {
    let path = project_path(name);

    if path.exists() {
        eprintln!("✗ 專案 '{}' 已存在: {}", name, path.display());
        return;
    }

    // 建立目錄結構
    let dirs = [
        "nodes",
        "output",
        "skills",
        "logs",
    ];

    println!("建立專案: {}", name);
    println!("路徑: {}", path.display());

    // 建立根目錄
    fs::create_dir_all(&path).unwrap_or_else(|e| {
        eprintln!("✗ 無法建立專案目錄: {}", e);
        std::process::exit(1);
    });

    // 建立子目錄
    for dir in &dirs {
        let sub = path.join(dir);
        fs::create_dir_all(&sub).unwrap_or_else(|e| {
            eprintln!("✗ 無法建立 {} 目錄: {}", dir, e);
            std::process::exit(1);
        });
    }

    // 建立 evolution.yaml
    let config_content = format!(
        r#"# Evolution OS 專案設定
name: {}
created: "{}"
task: ""
description: ""

# 使用的技能
skills: []

# Planner 設定
planner:
  work_mode: auto
  complexity_threshold: 5

# Executor 設定
executor:
  max_tier: 10
"#,
        name,
        chrono::Utc::now().to_rfc3339()
    );

    let config_path = path.join("evolution.yaml");
    fs::write(&config_path, config_content).unwrap_or_else(|e| {
        eprintln!("✗ 無法寫入設定檔: {}", e);
        std::process::exit(1);
    });

    // 建立 manifest.ev（空白藍圖）
    let manifest_path = path.join("manifest.ev");
    fs::write(&manifest_path, "# Evolution OS Manifest\n# 由 Planner自動產生\n").unwrap_or_else(|e| {
        eprintln!("✗ 無法寫入藍圖檔: {}", e);
        std::process::exit(1);
    });

    println!();
    println!("✓ 專案建立完成");
    println!();
    println!("  位置: {}", path.display());
    println!("  設定: {}", config_path.display());
    println!("  藍圖: {}", manifest_path.display());
    println!();
    println!("下一步：");
    println!("  evolution analyze \"你的任務描述\" --project {}", name);
}