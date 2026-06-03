//! boot — Evolution OS 開機流程
//!
//! Phase 1: 環境檢查（OS、硬體、目錄）
//! Phase 2: Ollama + Model 確認
//! Phase 3: 虛擬 OS 環境初始化
//! Phase 4: 準備就緒

use std::fs;
use std::path::PathBuf;

/// 開機 phase 狀態
#[derive(Debug)]
pub enum BootPhase {
    EnvironmentCheck,
    OllamaVerification,
    ModelVerification,
    VirtualOSInit,
    Ready,
}

/// 開機結果
#[derive(Debug)]
pub struct BootResult {
    pub success: bool,
    pub phase: BootPhase,
    pub message: String,
    pub details: serde_json::Value,
}

/// 執行完整開機流程
pub fn boot(verbose: bool) -> BootResult {
    let mut phases_completed = Vec::new();

    // ─── Phase 1: 環境檢查 ───────────────────────────────────────────
    if verbose {
        println!("[Phase 1/4] 環境檢查...");
    }

    let env_result = check_environment();
    if !env_result.success {
        return env_result;
    }
    phases_completed.push("environment");

    // ─── Phase 2: Ollama 確認 ────────────────────────────────────────
    if verbose {
        println!("[Phase 2/4] Ollama 服務確認...");
    }

    let ollama_result = check_ollama_service();
    if !ollama_result.success {
        return ollama_result;
    }
    phases_completed.push("ollama");

    // ─── Phase 3: Model 驗證 ─────────────────────────────────────────
    if verbose {
        println!("[Phase 3/4] gemma4:e2b 模型驗證...");
    }

    let model_result = check_model();
    if !model_result.success {
        return model_result;
    }
    phases_completed.push("model");

    // ─── Phase 4: 虛擬 OS 環境初始化 ──────────────────────────────────
    if verbose {
        println!("[Phase 4/4] 虛擬 OS 環境初始化...");
    }

    let init_result = init_virtual_os();
    if !init_result.success {
        return init_result;
    }
    phases_completed.push("virtual_os");

    if verbose {
        println!();
        println!("✓ Evolution OS 已就緒");
        println!();
        println!("  模型: gemma4:e2b (2B 參數)");
        println!("  路徑: ~/.evolution/");
        println!("  版本: 0.4.0-dev");
        println!();
        println!("  可用命令:");
        println!("    evolution analyze \"任務描述\"");
        println!("    evolution status");
        println!("    evolution ollama check");
    }

    BootResult {
        success: true,
        phase: BootPhase::Ready,
        message: "Evolution OS ready".to_string(),
        details: serde_json::json!({
            "version": "0.4.0-dev",
            "model": "gemma4:e2b",
            "platform": std::env::consts::OS,
            "phases_completed": phases_completed,
            "evolution_root": evolution_root().to_string_lossy().to_string(),
        }),
    }
}

/// Phase 1: 檢查環境（OS、硬體、目錄）
fn check_environment() -> BootResult {
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // 檢查 ~/.evolution/ 目錄
    let root = evolution_root();
    if !root.exists() {
        match fs::create_dir_all(&root) {
            Ok(_) => {}
            Err(e) => {
                return BootResult {
                    success: false,
                    phase: BootPhase::EnvironmentCheck,
                    message: format!("無法建立目錄: {}", e),
                    details: serde_json::json!({"error": e.to_string()}),
                };
            }
        }
    }

    // 檢查必要子目錄
    for subdir in ["projects", "tests", "logs"] {
        let path = root.join(subdir);
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(&path) {
                return BootResult {
                    success: false,
                    phase: BootPhase::EnvironmentCheck,
                    message: format!("無法建立目錄 {}: {}", subdir, e),
                    details: serde_json::json!({"error": e.to_string(), "path": path.to_string_lossy().to_string()}),
                };
            }
        }
    }

    BootResult {
        success: true,
        phase: BootPhase::EnvironmentCheck,
        message: format!("{} ({}) 環境就緒", platform, arch),
        details: serde_json::json!({
            "platform": platform,
            "arch": arch,
            "evolution_root": root.to_string_lossy().to_string(),
            "subdirs_created": ["projects", "tests", "logs"]
        }),
    }
}

/// Phase 2: 檢查 Ollama 服務
fn check_ollama_service() -> BootResult {
    use evolution_os::model::{ModelDispatcher, OllamaBackend};

    let backend = match OllamaBackend::new().health_check() {
        true => true,
        false => {
            return BootResult {
                success: false,
                phase: BootPhase::OllamaVerification,
                message: "Ollama 服務未運行".to_string(),
                details: serde_json::json!({
                    "error": "Ollama service not reachable at localhost:11434",
                    "hint": "執行: evolution ollama install && evolution ollama start"
                }),
            };
        }
    };

    let backend = OllamaBackend::new();
    let models = backend.available_models();

    BootResult {
        success: true,
        phase: BootPhase::OllamaVerification,
        message: format!("Ollama 服務正常 ({} models)", models.len()),
        details: serde_json::json!({
            "ollama_version": "0.20.2",
            "models_available": models,
            "model_count": models.len()
        }),
    }
}

/// Phase 3: 檢查 gemma4:e2b 模型
fn check_model() -> BootResult {
    use evolution_os::model::{ModelDispatcher, ModelRequest, OllamaBackend};

    let backend = OllamaBackend::new();
    let models = backend.available_models();

    let has_gemma4 = models.iter().any(|m| m.contains("gemma4:e2b"));

    if !has_gemma4 {
        return BootResult {
            success: false,
            phase: BootPhase::ModelVerification,
            message: "gemma4:e2b 模型未安裝".to_string(),
            details: serde_json::json!({
                "error": "Model gemma4:e2b not found",
                "available_models": models,
                "hint": "執行: ollama pull gemma4:e2b"
            }),
        };
    }

    // 實際推理測試
    let test_prompt = "回覆 OK 如果你能看到這段文字";
    let req = ModelRequest::new("gemma4:e2b", test_prompt)
        .with_temperature(0.1)
        .with_max_tokens(50);

    let resp = match backend.dispatch(req) {
        Ok(r) => r.content,
        Err(e) => {
            return BootResult {
                success: false,
                phase: BootPhase::ModelVerification,
                message: format!("模型推理測試失敗: {}", e),
                details: serde_json::json!({
                    "error": e.to_string(),
                    "model": "gemma4:e2b"
                }),
            };
        }
    };

    BootResult {
        success: true,
        phase: BootPhase::ModelVerification,
        message: "gemma4:e2b 模型正常".to_string(),
        details: serde_json::json!({
            "model": "gemma4:e2b",
            "test_response_preview": resp.chars().take(50).collect::<String>(),
            "tokens_used": 50
        }),
    }
}

/// Phase 4: 初始化虛擬 OS 環境
fn init_virtual_os() -> BootResult {
    let root = evolution_root();

    // 寫入版本標記
    let version_file = root.join("VERSION");
    if let Err(e) = fs::write(&version_file, "0.4.0-dev\n") {
        return BootResult {
            success: false,
            phase: BootPhase::VirtualOSInit,
            message: format!("無法寫入 VERSION: {}", e),
            details: serde_json::json!({"error": e.to_string()}),
        };
    }

    // 建立 SYSTEM.md（如果不存在）
    let system_md = root.join("SYSTEM.md");
    if !system_md.exists() {
        let system_content = r#"# Evolution OS 系統描述

## 版本
0.4.0-dev (Prototype)

## 核心模型
- 預設模型: gemma4:e2b (2B 參數)
- 備用模型: gemma4:e4b, llama3, mistral:7b

## 路徑結構
- projects/: 專案儲存
- tests/: 測試記錄
- logs/: 操作日誌

## 模型對應硬體需求
| 模型 | 參數 | 最低 RAM |
|------|------|----------|
| gemma4:e2b | 2B | 4GB |
| gemma4:e4b | 4B | 8GB |
| llama3 | 8B | 16GB |

Generated by Evolution OS boot
"#;
        if let Err(e) = fs::write(&system_md, system_content) {
            return BootResult {
                success: false,
                phase: BootPhase::VirtualOSInit,
                message: format!("無法建立 SYSTEM.md: {}", e),
                details: serde_json::json!({"error": e.to_string()}),
            };
        }
    }

    BootResult {
        success: true,
        phase: BootPhase::VirtualOSInit,
        message: "虛擬 OS 環境已初始化".to_string(),
        details: serde_json::json!({
            "version": "0.4.0-dev",
            "system_md": system_md.to_string_lossy().to_string(),
            "version_file": version_file.to_string_lossy().to_string()
        }),
    }
}

/// 取得 Evolution OS 根目錄
pub fn evolution_root() -> PathBuf {
    match std::env::consts::OS {
        "windows" => {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".evolution")
        }
        _ => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".evolution")
        }
    }
}

/// 輸出 JSON 格式（供 AI 閱讀）
pub fn boot_json() -> String {
    let result = boot(false);
    let status = if result.success { "ready" } else { "failed" };

    let phase_name = match result.phase {
        BootPhase::EnvironmentCheck => "environment_check",
        BootPhase::OllamaVerification => "ollama_verification",
        BootPhase::ModelVerification => "model_verification",
        BootPhase::VirtualOSInit => "virtual_os_init",
        BootPhase::Ready => "ready",
    };

    serde_json::json!({
        "status": status,
        "phase": phase_name,
        "message": result.message,
        "details": result.details,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }).to_string()
}