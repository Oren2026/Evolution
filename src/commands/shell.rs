//! shell — Evolution OS 自然語言對話介面
//!
//! 使用 gemma4:e2b 做 Intent Classification + 回覆生成
//! 不要求特定格式，使用者可用中英文自由輸入
//!
//! 意圖分類：
//!   - system: 開啟檔案、執行 CLI、開啟應用（真正執行 open）
//!   - evolution: analyze / new / status / list-skills / init（真正呼叫 CLI）
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
        "輸入：「{}」\n\n分類（只回一個英文單字）：\nsystem / evolution / calendar / general",
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

    // 解析 YAML
    let path = extract_yaml_field(&resp, "路徑");
    let target = extract_yaml_field(&resp, "目標");

    if path.is_empty() && target.is_empty() {
        return "我不太確定你要開啟什麼，可以說具體一點嗎？\n例如：「開啟 Safari」或「執行 node --version」".to_string();
    }

    // 根據關鍵字偵測應用類型
    let target_lower = target.to_lowercase();

    // 遊戲類：直接執行
    if target_lower.contains("遊戲") || target_lower.contains("game") || target_lower.contains("minecraft") {
        if target_lower.contains("minecraft") {
            return exec_open("/Applications/Minecraft.app");
        }
        return "我找不到這個遊戲，請告訴我完整的名稱或路徑".to_string();
    }

    // 一般開檔/執行命令
    let exec_path = if !path.is_empty() {
        path.clone()
    } else {
        target.clone()
    };

    // 如果是相對路徑或檔名，尝试扩张
    let final_path = if exec_path.starts_with("/") || exec_path.starts_with("~/") {
        exec_path
    } else if exec_path.contains(".") {
        // 可能是檔案，相對路徑
        std::env::current_dir()
            .map(|p| p.join(&exec_path).to_string_lossy().to_string())
            .unwrap_or(exec_path)
    } else {
        // 假設是應用程式名
        format!("/Applications/{}.app", exec_path)
    };

    exec_open(&final_path)
}

/// 執行 open 命令並回報結果
fn exec_open(path: &str) -> String {
    // 處理 ~ 擴展
    let expanded = if path.starts_with("~/") {
        dirs::home_dir()
            .map(|h| h.join(path.trim_start_matches("~/")))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string())
    } else {
        path.to_string()
    };

    match std::process::Command::new("open").arg(&expanded).spawn() {
        Ok(_) => format!("✅ 已執行：open \"{}\"", expanded),
        Err(e) => {
            // 尝试直接執行命令
            match std::process::Command::new("sh").arg("-c").arg(&expanded).spawn() {
                Ok(_) => format!("✅ 已執行：{}", expanded),
                Err(_) => format!("❌ 無法執行：{}\n原因：{}", expanded, e),
            }
        }
    }
}

/// Evolution: 自然語言驅動，用真正的命令
fn handle_evolution(backend: &OllamaBackend, input: &str) -> String {
    // 用 LLM 理解使用者想要做什麼 Evolution 操作
    let prompt = format!(
        r#"輸入：「{}」

这是 Evolution OS 的指令。判斷使用者想要：
1. 分析任務（analyze）- 「分析XX」「帮我看XX」「检视XX」
2. 建立專案（new）- 「建立專案」「新專案」「創建」
3. 查看狀態（status）- 「系統狀態」「目前怎樣」
4. 列出技能（list-skills）- 「有什麼技能」「技能列表」
5. 初始化（init）- 「初始化」「生成概述」
6. 開啟 shell（shell）- 「進入對話模式」「開始聊天」

提取：
- 操作類型
- 参數（如專案名稱或任務描述）

只回 YAML：
操作：「」
参数：「」
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

    let op = extract_yaml_field(&resp, "操作");
    let param = extract_yaml_field(&resp, "参数");

    match op.as_str() {
        "analyze" => {
            if param.is_empty() {
                "🔍 要分析什麼？\n例如：「帮我分析這個資料夾的結構」或「分析：建立一個計數器」".to_string()
            } else {
                // 真正執行 analyze
                let output = std::process::Command::new("cargo")
                    .args(["run", "--", "analyze", &param])
                    .current_dir("/Users/oren/Desktop/Evolution-new")
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if stdout.is_empty() && stderr.is_empty() {
                            "✅ 分析完成（無輸出）".to_string()
                        } else if out.status.success() {
                            format!("✅ 分析完成\n{}", stdout.trim())
                        } else {
                            format!("⚠️ 分析完成但有警告：\n{}\n{}", stderr.trim(), stdout.trim())
                        }
                    }
                    Err(e) => format!("❌ 無法執行分析：{}\n提示：先確認任務描述是否完整", e),
                }
            }
        }
        "new" => {
            if param.is_empty() {
                "📁 新專案要叫什麼名字？\n例如：「新專案叫電商系統」".to_string()
            } else {
                // 真正執行 new_project
                let output = std::process::Command::new("cargo")
                    .args(["run", "--", "new", &param])
                    .current_dir("/Users/oren/Desktop/Evolution-new")
                    .output();

                match output {
                    Ok(out) => {
                        if out.status.success() {
                            format!("✅ 專案「{}」建立完成", param)
                        } else {
                            format!("❌ 建立失敗：{}", String::from_utf8_lossy(&out.stderr).trim())
                        }
                    }
                    Err(e) => format!("❌ 無法建立專案：{}", e),
                }
            }
        }
        "status" => {
            // 真正執行 status
            let output = std::process::Command::new("cargo")
                .args(["run", "--", "status"])
                .current_dir("/Users/oren/Desktop/Evolution-new")
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        String::from_utf8_lossy(&out.stdout).trim().to_string()
                    } else {
                        format!("❌ 狀態查詢失敗：{}", String::from_utf8_lossy(&out.stderr).trim())
                    }
                }
                Err(e) => format!("❌ 無法查詢狀態：{}", e),
            }
        }
        "list-skills" => {
            // 真正執行 list_skills
            let output = std::process::Command::new("cargo")
                .args(["run", "--", "list-skills"])
                .current_dir("/Users/oren/Desktop/Evolution-new")
                .output();

            match output {
                Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
                Err(e) => format!("❌ 無法列出技能：{}", e),
            }
        }
        "init" => {
            let proj = if param.is_empty() { None } else { Some(param.as_str()) };
            crate::commands::init::init(proj);
            if param.is_empty() {
                "✅ 概述（系統）已生成".to_string()
            } else {
                format!("✅ 概述：{} 已生成", param)
            }
        }
        "shell" => {
            "🔄 切換到 shell 模式，請輸入 exit 回到一般對話".to_string()
        }
        _ => {
            if op.is_empty() {
                "🤔 我不太理解你要做什麼 Evolution 操作。\n可以說：「幫我分析這個任務」「建立新專案」「查看系統狀態」".to_string()
            } else {
                format!("⚠️ 不支援的 Evolution 操作：{}\n嘗試說：「幫我分析XX」「新專案叫XX」", op)
            }
        }
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

