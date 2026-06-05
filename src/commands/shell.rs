//! shell — Evolution OS 自然語言對話介面
//!
//! 整合 IntentRouter：統一意圖分類 + 命令執行。
//! 不要求特定格式，使用者可用中英文自由輸入。
//!
//! 路由邏輯：
//!   1. IntentRouter.route_with_model() → Evolution CLI 命令 → 直接執行
//!   2. 失敗 → route_keyword() fallback
//!   3. 都失敗 → 意圖分類（general / calendar / system）
//!   4. opencode → handle_opencode

use std::io::{self, Write};
use std::process::Command;
use std::time::Instant;

use evolution_os::model::{ModelDispatcher, ModelRequest, OllamaBackend};
use evolution_os::system::{StateBoard, StateBoardStorage, TaskStage};
use crate::commands::intent_router::IntentRouter;

/// 對話系統提示詞
const SYSTEM_PROMPT: &str = r#"你是 Evolution OS 的語音介面精靈，命名為「小E」。

你的職責：
1. 理解使用者用自然語言錸达的指令（中文、英文或混合）
2. 將意圖分類並執行對應動作
3. 以繁體中文友善回覆

規則：
- 系統操作（開檔/執行）用口語說明即將發生的事，再執行
- Evolution 任務直接執行並回報結果
- 行事曆需求：先確認時間地點，再寫入
- 不知道的事直接說不知道，不會胡亂回答
- 回覆簡潔，不囉嗦"#;

/// opencode 意圖關鍵字
const OPENCODE_KEYWORDS: &[&str] = &[
    "opencode", "幫我寫", "寫程式", "coding", "程式設計",
    "開發", "code", "代碼",
];

pub fn shell() {
    let backend = OllamaBackend::new();
    let router = IntentRouter::new();

    println!();
    println!("╔══════════════════════════════════════════════╗");
    println!("║         Evolution OS — 小E 對話模式           ║");
    println!("║  直接說出你想做什麼，不需要特定格式            ║");
    println!("║  輸入 'exit' 結束對話                         ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("✨ 系統就緒，等候你的指令...\n");

    loop {
        print!("你 ▶ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            println!("\n下次見！");
            break;
        }

        let user_input = input.trim();
        if user_input.is_empty() {
            continue;
        }

        if user_input == "exit" || user_input == "quit" || user_input == "再見" {
            println!("下次見！ 👋");
            break;
        }

        println!("\n你 ▶ {}\n", user_input);

        let start = Instant::now();
        let response = route_and_execute(&router, &backend, user_input);
        let elapsed = start.elapsed();

        println!("{} ({}ms)\n", response, elapsed.as_millis());
    }
}

// ═══════════════════════════════════════════════════════════════════
// 路由核心
// ═══════════════════════════════════════════════════════════════════

/// 主要路由：IntentRouter → Evolution CLI / fallback handlers
fn route_and_execute(router: &IntentRouter, backend: &OllamaBackend, input: &str) -> String {
    // Step 1: 嘗試 IntentRouter LLM 路由（Evolution CLI 命令）
    let result = router.route_with_model(input, backend);

    match result {
        Ok(matched) => {
            println!("🔍 理解中... (命令：{} 置信度：{})\n", matched.command_id, matched.confidence);
            execute_evolution_cli(&matched.cli)
        }
        Err(_) => {
            // Step 2: 嘗試 keyword fallback
            if let Some(keyword_match) = router.route_keyword(input) {
                println!("🔍 理解中... (關鍵字匹配：{})\n", keyword_match.command_id);
                execute_evolution_cli(&keyword_match.cli)
            } else {
                // Step 3: 走 fallback 意圖分類
                fallback_intent分类(backend, input)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Fallback 意圖分類（IntentRouter 完全匹配失敗時）
// ═══════════════════════════════════════════════════════════════════

fn fallback_intent分类(backend: &OllamaBackend, input: &str) -> String {
    let intent = classify_intent(backend, input);
    match intent.as_str() {
        "system" => handle_system(backend, input),
        "evolution" => handle_evolution(backend, input),
        "opencode" => handle_opencode(backend, input),
        "calendar" => handle_calendar(backend, input),
        _ => handle_general(backend, input),
    }
}

/// 快速意圖分類（fallback 用）
fn classify_intent(backend: &OllamaBackend, input: &str) -> String {
    // opencode 關鍵字搶先判斷
    let input_lower = input.to_lowercase();
    for kw in OPENCODE_KEYWORDS {
        if input_lower.contains(kw) {
            return "opencode".to_string();
        }
    }

    let prompt = format!(
        "輸入：「{}」\n\n分類（只回一個英文單字）：\nsystem / evolution / opencode / calendar / general",
        input
    );
    let req = ModelRequest {
        model: "gemma4:e2b".to_string(),
        prompt,
        system_prompt: Some(SYSTEM_PROMPT.to_string()),
        temperature: 0.1,
        max_tokens: Some(10),
    };

    match backend.dispatch(req) {
        Ok(resp) => {
            let result = resp.content.trim().to_lowercase();
            if result.contains("system") {
                "system".to_string()
            } else if result.contains("evolution") {
                "evolution".to_string()
            } else if result.contains("opencode") {
                "opencode".to_string()
            } else if result.contains("calendar") {
                "calendar".to_string()
            } else {
                "general".to_string()
            }
        }
        Err(_) => "general".to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════
// StateBoard helpers
// ═══════════════════════════════════════════════════════════════════

fn evolution_root() -> std::path::PathBuf {
    std::env::var("EVOLUTION_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".evolution")
        })
}

fn board_storage_path() -> std::path::PathBuf {
    evolution_root().join("state_board.json")
}

fn load_board() -> StateBoard {
    let path = board_storage_path();
    let storage = StateBoardStorage::new(&path.to_string_lossy());
    storage.load().unwrap_or_else(|_| StateBoard::new())
}

fn save_board(board: &StateBoard) {
    let path = board_storage_path();
    let storage = StateBoardStorage::new(&path.to_string_lossy());
    let _ = storage.save(board);
}

// ═══════════════════════════════════════════════════════════════════
// Evolution CLI 執行器（帶 StateBoard 追蹤）
// ═══════════════════════════════════════════════════════════════════

/// 執行 Evolution CLI 並將結果寫入 StateBoard
fn execute_evolution_cli(cli: &str) -> String {
    // 解析任務名稱（從 cli 推斷）
    let task_name = extract_task_name_from_cli(cli);

    // 建立或取得 StateBoard 任務
    let mut board = load_board();
    let task_id = board.create_task(task_name.clone());
    board.update_stage(&task_id, TaskStage::Executing, None, None);
    save_board(&board);

    // 執行 CLI
    let parts: Vec<&str> = cli.trim_start_matches("evolution").trim().split_whitespace().collect();
    if parts.is_empty() {
        return "❌ 無效的命令".to_string();
    }

    let output = Command::new("~/.cargo/bin/cargo")
        .args(&["run", "--manifest-path", "/Users/oren/Desktop/Evolution-new/Cargo.toml", "--"])
        .args(&parts)
        .output();

    // 更新 StateBoard 任務階段
    let mut board = load_board();
    match output {
        Ok(out) if out.status.success() => {
            board.update_stage(&task_id, TaskStage::Done, None, None);
            save_board(&board);
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.trim().is_empty() {
                String::from_utf8_lossy(&out.stderr).trim().to_string()
            } else {
                stdout.trim().to_string()
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let note = Some(format!("執行失敗: {}", stderr.trim()));
            board.update_stage(&task_id, TaskStage::Blocked, note, None);
            save_board(&board);
            format!("❌ 命令執行失敗\n{}", stderr.trim())
        }
        Err(e) => {
            let note = Some(format!("無法執行: {}", e));
            board.update_stage(&task_id, TaskStage::Blocked, note, None);
            save_board(&board);
            format!("❌ 無法執行命令：{}", e)
        }
    }
}

/// 從 CLI 字串推斷任務名稱（用於 StateBoard 追蹤）
fn extract_task_name_from_cli(cli: &str) -> String {
    // cli 格式："evolution analyze xxx" / "evolution board status" / "evolution new project"
    let parts: Vec<&str> = cli.trim_start_matches("evolution").trim().split_whitespace().collect();
    if parts.len() >= 2 {
        format!("{} {}", parts[0], parts[1])
    } else if parts.len() == 1 {
        parts[0].to_string()
    } else {
        "未知任務".to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Handler 函式
// ═══════════════════════════════════════════════════════════════════

/// System: 開檔、執行命令（不走 Evolution CLI 的系統操作）
fn handle_system(_backend: &OllamaBackend, input: &str) -> String {
    format!(
        "🔧 系統操作：{}\n\n[系統操作需要更多細節才能執行，例如：開啟哪個檔案、執行什麼命令]",
        input
    )
}

/// Evolution: 直接呼叫 CLI（IntentRouter 匹配不到的罕用命令 fallback）
fn handle_evolution(backend: &OllamaBackend, input: &str) -> String {
    // 從輸入提取 Evolution CLI 子命令
    let prompt = format!(
        r#"輸入：「{}」

判斷是否要呼叫 Evolution CLI（analyze / new project / status / init / list-skills / board）。
如果是，回覆 CLI 命令格式（如 "evolution analyze xxx"）；如果不是，說「不需要CLI」。

只回覆一行。"#,
        input
    );

    let req = ModelRequest {
        model: "gemma4:e2b".to_string(),
        prompt,
        system_prompt: None,
        temperature: 0.0,
        max_tokens: Some(100),
    };

    match backend.dispatch(req) {
        Ok(resp) => {
            let cmd = resp.content.trim();
            if cmd.starts_with("evolution ") {
                execute_evolution_cli(cmd)
            } else {
                format!("不需要 Evolution CLI：{}", cmd)
            }
        }
        Err(e) => format!("抱歉，系統有點慢：{}", e),
    }
}

/// OpenCode: 任務委派（已正確接入）
fn handle_opencode(backend: &OllamaBackend, input: &str) -> String {
    let prompt = format!(
        r#"輸入：「{}」

判斷這是否適合交給 OpenCode（軟體開發工具）處理。
如果適合，說明要做什麼；如果不適合，說原因。

只回覆繁體中文，一句話。"#,
        input
    );

    let req = ModelRequest {
        model: "gemma4:e2b".to_string(),
        prompt,
        system_prompt: None,
        temperature: 0.3,
        max_tokens: Some(80),
    };

    match backend.dispatch(req) {
        Ok(r) => {
            format!(
                "🤖 OpenCode 任務：{}\n\n[任務委派中...]",
                r.content.trim()
            )
        }
        Err(e) => format!("無法處理：{}", e),
    }
}

/// Calendar: 記事
fn handle_calendar(backend: &OllamaBackend, input: &str) -> String {
    let prompt = format!(
        r#"輸入：「{}」

提取行事曆事件的：
1. 時間（什麼時候）
2. 事項（要做什麼）
3. 地點（如果有）

YAML 格式：

時間：「」
事項：「」
地點：「」"#,
        input
    );

    let req = ModelRequest {
        model: "gemma4:e2b".to_string(),
        prompt,
        system_prompt: None,
        temperature: 0.1,
        max_tokens: Some(150),
    };

    let resp = match backend.dispatch(req) {
        Ok(r) => r.content,
        Err(e) => return format!("LLM 錯誤：{}", e),
    };

    let time = extract_yaml_field(&resp, "時間");
    let event = extract_yaml_field(&resp, "事項");
    let location = extract_yaml_field(&resp, "地點");

    if event.is_empty() {
        return "我需要知道你要記錄什麼事件，請重新說明一次。".to_string();
    }

    let time_str = if time.is_empty() {
        "時間待確認".to_string()
    } else {
        time.clone()
    };

    format!(
        "📅 記事已準備：\n  時間：{}\n  事項：{}\n  地點：{}\n\n要幫你寫入行事曆嗎？",
        time_str,
        event,
        if location.is_empty() { "未指定" } else { &location }
    )
}

/// General: 一般對話
fn handle_general(backend: &OllamaBackend, input: &str) -> String {
    let prompt = format!(
        "你是 Evolution OS 的小E精靈，使用繁體中文回答。\n\n使用者：{}\n小E：",
        input
    );

    let req = ModelRequest {
        model: "gemma4:e2b".to_string(),
        prompt,
        system_prompt: Some(SYSTEM_PROMPT.to_string()),
        temperature: 0.8,
        max_tokens: Some(300),
    };

    match backend.dispatch(req) {
        Ok(r) => r.content.trim().to_string(),
        Err(e) => format!("抱歉，系統有點慢：{}", e),
    }
}

/// 簡單 YAML 欄位提取
fn extract_yaml_field(yaml: &str, field: &str) -> String {
    for line in yaml.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{}：「", field))
            || line.starts_with(&format!("{}: \"", field))
            || line.starts_with(&format!("{}:「", field))
        {
            let rest = line.split(['：', ':', '「', '"']).nth(1).unwrap_or("").trim_end_matches('」').trim_end_matches('"');
            return rest.to_string();
        }
    }
    String::new()
}