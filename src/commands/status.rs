//! status — 系統狀態檢查

use std::iter;
use evolution_os::model::OllamaBackend;
use evolution_os::model::ModelDispatcher;
use std::process::Command;

/// 系統狀態資訊
pub struct SystemStatus {
    pub ollama_running: bool,
    pub ollama_version: Option<String>,
    pub models: Vec<String>,
    pub evolution_root: String,
    pub projects_count: usize,
}

impl SystemStatus {
    pub fn collect() -> Self {
        let ollama = OllamaBackend::new();
        let models = ollama.available_models();

        // 檢查 Ollama 版本
        let ollama_version = Command::new("ollama")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        // 檢查 Ollama 是否運行（嘗試 API）
        let ollama_running = !models.is_empty();

        // Evolution 根目錄
        let evolution_root = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".evolution")
            .to_string_lossy()
            .to_string();

        // 專案數量
        let projects_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".evolution")
            .join("projects");

        let projects_count = std::fs::read_dir(&projects_path)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count()
            })
            .unwrap_or(0);

        Self {
            ollama_running,
            ollama_version,
            models,
            evolution_root,
            projects_count,
        }
    }

    /// 以 JSON 格式輸出
    pub fn to_json(&self) -> String {
        let models_json = self
            .models
            .iter()
            .map(|m| format!("\"{}\"", m))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"{{
  "ollama": {{
    "running": {},
    "version": {},
    "models": [{}]
  }},
  "evolution": {{
    "root": "{}",
    "projects_count": {}
  }}
}}"#,
            self.ollama_running,
            self.ollama_version
                .as_ref()
                .map(|s| format!("\"{}\"", s))
                .unwrap_or("null".to_string()),
            models_json,
            self.evolution_root,
            self.projects_count
        )
    }
}

fn make_line(ch: &str, len: usize) -> String {
    iter::repeat(ch).take(len).collect()
}

/// 輸出狀態（可選 JSON）
pub fn status(json: bool) {
    let s = SystemStatus::collect();

    if json {
        println!("{}", s.to_json());
        return;
    }

    let line = make_line("=", 50);

    println!();
    println!("{}", line);
    println!("  Evolution OS 系統狀態");
    println!("{}", line);
    println!();

    // Ollama 狀態
    println!("📦 Ollama");
    if s.ollama_running {
        println!("  ✅ 運行中");
        if let Some(ref v) = s.ollama_version {
            println!("  版本：{}", v);
        }
        println!("  模型（{}）：", s.models.len());
        for model in &s.models {
            println!("    • {}", model);
        }
    } else {
        println!("  ❌ 未運行");
        println!("  安裝指令：brew install ollama  (macOS)");
        println!("              winget install Ollama (Windows)");
        println!("  或從 https://ollama.com 下載");
    }

    println!();
    println!("📁 Evolution");
    println!("  根目錄：{}", s.evolution_root);
    println!("  專案數：{}", s.projects_count);

    println!();
    println!("{}", line);
}
