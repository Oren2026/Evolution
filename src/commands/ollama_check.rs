//! ollama_check — Ollama 健康檢查 + 自動安裝
//!
//! 使用方式：
//!   evolution ollama check   檢查狀態（可加 --json）
//!   evolution ollama install  安裝預設模型（llama3）
//!   evolution ollama start    啟動 Ollama 服務

use evolution_os::model::OllamaBackend;
use evolution_os::model::ModelDispatcher;
use std::process::Command;

/// Ollama 狀態
pub struct OllamaStatus {
    pub running: bool,
    pub version: Option<String>,
    pub models: Vec<String>,
    pub install_command: String,
}

impl OllamaStatus {
    /// 檢查 Ollama 狀態
    pub fn check() -> Self {
        let backend = OllamaBackend::new();
        let models = backend.available_models();
        let running = !models.is_empty();

        // 取得版本
        let version = Command::new("ollama")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        // 根據平台給出安裝指令
        #[cfg(target_os = "windows")]
        let install_cmd = "winget install Ollama.Ollama".to_string();
        #[cfg(target_os = "macos")]
        let install_cmd = "brew install ollama".to_string();
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let install_cmd = "curl -fsSL https://ollama.com/install.sh | sh".to_string();

        Self {
            running,
            version,
            models,
            install_command: install_cmd,
        }
    }

    /// 嘗試啟動 Ollama（macOS 用 launchctl，Windows 用 start service）
    pub fn try_start(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // macOS: 嘗試用 open 啟動（會開啟一個新終端視窗）
            let output = Command::new("open")
                .arg("-a")
                .arg("Ollama")
                .output()
                .map_err(|e| format!("無法啟動 Ollama: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                Err("Ollama 未安裝，請先執行 install".to_string())
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: 嘗試啟動服務
            let output = Command::new("sc")
                .args(["start", "Ollama"])
                .output()
                .map_err(|e| format!("無法啟動 Ollama service: {}", e))?;

            if output.status.success() {
                Ok(())
            } else {
                Err("Ollama 未安裝，請先執行 install".to_string())
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            Err("請手動啟動 Ollama: ollama serve".to_string())
        }
    }

    /// 自動安裝 llama3
    pub fn install_default(&self) -> Result<(), String> {
        if self.running {
            return Ok(()); // 已經有模型運行，不需要安裝
        }

        println!("📦 安裝 llama3（預設模型）...");
        println!("  這可能需要幾分鐘，下載約 4.7GB...");

        let output = Command::new("ollama")
            .args(["pull", "llama3"])
            .output()
            .map_err(|e| format!("安裝失敗: {}", e))?;

        if output.status.success() {
            println!("  ✅ llama3 安裝完成");
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(format!("安裝失敗: {}", err))
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
    "models": [{}],
    "install_command": "{}"
  }}
}}"#,
            self.running,
            self.version
                .as_ref()
                .map(|s| format!("\"{}\"", s))
                .unwrap_or("null".to_string()),
            models_json,
            self.install_command
        )
    }
}

fn make_line(len: usize) -> String {
    std::iter::repeat('=').take(len).collect()
}

/// 檢查並回報狀態
pub fn check(json: bool) {
    let status = OllamaStatus::check();

    if json {
        println!("{}", status.to_json());
        return;
    }

    let line = make_line(50);

    println!();
    println!("{}", line);
    println!("  Ollama 狀態檢查");
    println!("{}", line);
    println!();

    if status.running {
        println!("  ✅ Ollama 運行中");
        if let Some(ref v) = status.version {
            println!("     版本：{}", v);
        }
        println!("     模型：{} 個", status.models.len());
        for model in &status.models {
            println!("       • {}", model);
        }
    } else {
        println!("  ❌ Ollama 未運行或未安裝");
        println!();
        println!("  安裝指令：");
        println!("    {}", status.install_command);
        println!();
        println!("  或從 https://ollama.com 下載");
    }

    println!();
    println!("{}", line);
}

/// 安裝預設模型
pub fn install() {
    let status = OllamaStatus::check();

    if status.running && !status.models.is_empty() {
        println!("✅ Ollama 已有模型運行，跳過安裝");
        println!("  模型：{:?}", status.models);
        return;
    }

    match status.install_default() {
        Ok(()) => {
            println!("✅ 安裝完成");
        }
        Err(e) => {
            eprintln!("✗ {}", e);
            std::process::exit(1);
        }
    }
}

/// 啟動 Ollama
pub fn start() {
    let status = OllamaStatus::check();

    if status.running {
        println!("✅ Ollama 已在運行");
        return;
    }

    print!("啟動 Ollama...");
    match status.try_start() {
        Ok(()) => {
            println!(" 完成");
            println!();
            println!("  等待 3 秒後檢查狀態...");
            std::thread::sleep(std::time::Duration::from_secs(3));

            let new_status = OllamaStatus::check();
            if new_status.running {
                println!("  ✅ Ollama 已啟動");
                println!("  模型：{:?}", new_status.models);
            } else {
                println!("  ⚠️  啟動後無法連線，請手動啟動");
            }
        }
        Err(e) => {
            println!();
            eprintln!("✗ {}", e);
            println!();
            println!("  請先安裝：");
            println!("    {}", status.install_command);
        }
    }
}

/// Evolution System Call — 包裝 dispatch() 為系統呼叫
/// 這是 Evolution OS 的 LLM 整合核心
pub mod syscall {
    use evolution_os::model::{DispatchError, ModelDispatcher, ModelRequest};

    /// 發送 System Call 到 LLM（透過 Ollama）
    pub fn llm_call(prompt: &str) -> Result<String, String> {
        let backend = evolution_os::model::OllamaBackend::new();
        let models = backend.available_models();

        if models.is_empty() {
            return Err("Ollama 未運行，請先執行: evolution ollama install".to_string());
        }

        // 預設使用第一個可用模型
        let model = models.first().map(|s| s.as_str()).unwrap_or("llama3");
        let req = ModelRequest::new(model, prompt);

        backend
            .dispatch(req)
            .map(|r| r.content)
            .map_err(|e| format!("LLM call failed: {:?}", e))
    }

    /// 發送到指定模型
    #[allow(dead_code)]
    pub fn llm_call_with_model(model: &str, prompt: &str) -> Result<String, String> {
        let backend = evolution_os::model::OllamaBackend::new();
        let req = ModelRequest::new(model, prompt);

        backend
            .dispatch(req)
            .map(|r| r.content)
            .map_err(|e| format!("LLM call failed: {:?}", e))
    }

    /// 列出可用模型（System Call）
    pub fn list_models() -> Vec<String> {
        let backend = evolution_os::model::OllamaBackend::new();
        backend.available_models()
    }

    /// 健康檢查（System Call）
    pub fn health_check() -> bool {
        let backend = evolution_os::model::OllamaBackend::new();
        !backend.available_models().is_empty()
    }
}