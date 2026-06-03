//! shell — Evolution OS 自然語言對話介面
//!
//! 使用 gemma4:e2b 做 Intent Classification + 回覆生成
//! 不要求特定格式，使用者可用中英文自由輸入
//!
//! 意圖分類：
//!   - system: 開啟檔案、執行 CLI、開啟應用
//!   - evolution: analyze / new project / 專案操作
//!   - opencode: 任務委派給 OpenCode
//!   - calendar: 行事曆事件記錄
//!   - general: 一般問答

use std::io::{self, Write};
use std::time::Instant;

use evolution_os::model::{ModelDispatcher, ModelRequest, OllamaBackend};

/// 對話系統提示詞
const SYSTEM_PROMPT: &str = r#"你是 Evolution OS 的語音介面精靈，命名為「小E」。

你的職責：
1. 理解使用者用自然語言錸达的指令（中文、英文或混合）
2. 將意圖分類並執行對應動作
3. 以繁體中文友善回覆

意圖分類：
- system：開啟檔案/執行命令/開啟應用
- evolution：使用 Evolution OS 的指令（analyze/new project等）
- opencode：需要 OpenCode 處理的任務
- calendar：需要記錄到行事曆的事件
- general：一般問題或閒聊

規則：
- 系統操作（開檔/執行）用口語說明即將發生的事，再執行
- Evolution 任務直接執行並回報結果
- 行事曆需求：先確認時間地點，再寫入
- 不知道的事直接說不知道，不會胡亂回答
- 回覆簡潔，不囉嗦"#;

/// 快速分類 prompt（少 token，節省時間）
const CLASSIFY_PROMPT: &str = r#"輸入：「{}」

分類（只回一個英文單字）：
system / evolution / opencode / calendar / general"#;

pub fn shell() {
    let backend = OllamaBackend::new();

    println!();
    println!("╔══════════════════════════════════════════════╗");
    println!("║         Evolution OS — 小E 對話模式           ║");
    println!("║  直接說出你想做什麼，不需要特定格式            ║");
    println!("║  輸入 'exit' 結束對話                         ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("✨ 系統就緒，等候你的指令...\n");

    let mut session = Vec::new();

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

        // 印出使用者輸入
        println!("\n你 ▶ {}\n", user_input);
        session.push(format!("使用者：{}", user_input));

        let start = Instant::now();

        // Step 1: Intent Classification
        let intent = classify_intent(&backend, user_input);
        println!("🔍 理解中... (意圖：{})\n", intent);

        // Step 2: Execute based on intent
        let response = execute(&backend, user_input, &intent);

        let elapsed = start.elapsed();
        println!("{} ({}ms)\n", response, elapsed.as_millis());

        session.push(format!("小E：{}", response));
    }
}

/// 意圖分類
fn classify_intent(backend: &OllamaBackend, input: &str) -> String {
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

/// 根據意圖執行
fn execute(backend: &OllamaBackend, input: &str, intent: &str) -> String {
    match intent {
        "system" => handle_system(backend, input),
        "evolution" => handle_evolution(backend, input),
        "opencode" => handle_opencode(backend, input),
        "calendar" => handle_calendar(backend, input),
        _ => handle_general(backend, input),
    }
}

/// System: 開檔、執行命令、遊戲
fn handle_system(backend: &OllamaBackend, input: &str) -> String {
    // 用 LLM 解讀使用者想要做什麼系統操作
    let prompt = format!(
        r#"輸入：「{}」

這個操作涉及開啟檔案、執行命令、或啟動應用程式嗎？
如果是的話，提取：
1. 目標（要做什麼）
2. 具體路徑或名稱（如果有）

只回覆 YAML 格式，無其他文字：

目標：「」
路徑：「」
"#,
        input
    );

    let req = ModelRequest {
        model: "gemma4:e2b".to_string(),
        prompt,
        system_prompt: None,
        temperature: 0.1,
        max_tokens: Some(100),
    };

    let resp = match backend.dispatch(req) {
        Ok(r) => r.content,
        Err(e) => return format!("LLM 錯誤：{}", e),
    };

    // 簡單解析 YAML（找「路徑」或「目標」）
    let path = extract_yaml_field(&resp, "路徑");
    let target = extract_yaml_field(&resp, "目標");

    if path.is_empty() && target.is_empty() {
        return "我不太確定你要開啟什麼，可以說具體一點嗎？\n例如：「開啟 Safari」或「執行 node --version」".to_string();
    }

    let cmd = if !path.is_empty() {
        format!("open \"{}\"", path)
    } else {
        target.clone()
    };

    // 檢查是否是常見應用
    if target.contains("遊戲") || target.contains("game") {
        let game_path = find_game(&target);
        if let Some(p) = game_path {
            return format!("啟動遊戲：{} ... 執行中 🎮", p);
        }
    }

    format!("即將執行：{}\n（系統指令）", cmd)
}

/// Evolution: analyze / new project
fn handle_evolution(backend: &OllamaBackend, input: &str) -> String {
    let prompt = format!(
        r#"輸入：「{}」

這是 Evolution OS 的指令，提取：
1. 指令類型（analyze / new / status / list-skills / init）
2. 參數（任務描述 或 專案名稱）

YAML 格式：

指令：「」
參數：「」
"#,
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

    let cmd = extract_yaml_field(&resp, "指令");
    let param = extract_yaml_field(&resp, "參數");

    match cmd.as_str() {
        "analyze" => {
            if param.is_empty() {
                "分析任務需要具體描述，例如：「分析：建立一個計數器網頁」".to_string()
            } else {
                format!("🚀 啟動 Evolution 分析：{}\n\n[分析引擎啟動中...]", param)
            }
        }
        "new" => {
            if param.is_empty() {
                "新專案需要名稱，例如：「新專案叫電商系統」".to_string()
            } else {
                format!("📁 建立新專案：{}\n\n[專案建立中...]", param)
            }
        }
        "status" => "🔍 系統狀態檢查中...\n".to_string(),
        "list-skills" => "📋 技能列表讀取中...\n".to_string(),
        "init" => {
            if param.is_empty() {
                format!("📄 初始化系統概述")
            } else {
                format!("📄 初始化專案概述：{}", param)
            }
        }
        _ => format!("收到 Evolution 指令：{}（參數：{}）\n[準備執行中...]", cmd, param),
    }
}

/// OpenCode: 任務委派
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
地點：「」
"#,
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
        r#"你是 Evolution OS 的小E精靈，使用繁體中文回答。

使用者：{}
小E："#,
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
            // 擷取引號或「」之間的內容
            let rest = line.split(['：', ':', '「', '"']).nth(1).unwrap_or("").trim_end_matches('」').trim_end_matches('"');
            return rest.to_string();
        }
    }
    String::new()
}

/// 找遊戲路徑（簡單版）
fn find_game(name: &str) -> Option<String> {
    let name_lower = name.to_lowercase();

    // 常見遊戲
    let games = [
        ("minecraft", "/Applications/Minecraft.app"),
        ("safari", "/Applications/Safari.app"),
        ("terminal", "/Applications/Utilities/Terminal.app"),
    ];

    for (keyword, path) in &games {
        if name_lower.contains(keyword) {
            return Some(path.to_string());
        }
    }

    None
}