//! Intent Command DB — 意圖命令資料庫
//!
//! 将用户自然语言映射到系统内部命令的数据库。
//! 由 IntentRouter 使用本地模型进行意图分类。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// 資料模型
// ═══════════════════════════════════════════════════════════════════

/// 單一命令的參數槽位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSlot {
    /// 槽位名稱（如 "task_description"）
    pub name: String,
    /// 描述這個參數的用途
    pub description: String,
    /// 是否必填
    pub required: bool,
}

/// 單一命令定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDef {
    /// 命令唯一 ID
    pub id: String,
    /// 所屬類別
    pub category: String,
    /// 別名（用于 aliases 比對，或直接描述）
    pub aliases: Vec<String>,
    /// 命令描述
    pub description: String,
    /// 實際執行的 CLI 命令（不含 evolution 前綴）
    pub cli: String,
    /// 參數槽位定義
    pub slots: Vec<CommandSlot>,
    /// 使用範例（用於訓練或提示）
    pub examples: Vec<String>,
}

impl CommandDef {
    /// 檢查是否匹配某個別名或描述關鍵字
    pub fn matches(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        // 檢查 aliases
        for alias in &self.aliases {
            if input_lower.contains(&alias.to_lowercase()) {
                return true;
            }
        }
        // 檢查 description（關鍵字比對）
        let desc_words: Vec<&str> = self.description.split([' ', '，', '。']).collect();
        for word in desc_words {
            if word.len() > 1 && input_lower.contains(&word.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

/// IntentCommandDB 本體
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCommandDB {
    pub version: String,
    pub categories: Vec<String>,
    pub commands: Vec<CommandDef>,
}

impl IntentCommandDB {
    /// 建立帶有內建命令的資料庫
    pub fn new() -> Self {
        Self {
            version: "0.1.0".to_string(),
            categories: vec![
                "system".into(),
                "planner".into(),
                "board".into(),
                "dev".into(),
                "query".into(),
            ],
            commands: builtin_commands(),
        }
    }

    /// 依 ID 查找命令
    pub fn get(&self, id: &str) -> Option<&CommandDef> {
        self.commands.iter().find(|c| c.id == id)
    }

    /// 列出某類別的所有命令
    pub fn by_category(&self, cat: &str) -> Vec<&CommandDef> {
        self.commands.iter().filter(|c| c.category == cat).collect()
    }

    /// 根據輸入找到最匹配的命令
    pub fn find_best_match(&self, input: &str) -> Option<&CommandDef> {
        // 先用精確 matching
        for cmd in &self.commands {
            if cmd.matches(input) {
                return Some(cmd);
            }
        }
        None
    }

    /// 取得所有命令 ID
    pub fn command_ids(&self) -> Vec<&str> {
        self.commands.iter().map(|c| c.id.as_str()).collect()
    }

    /// 產生 CLI 字串（含前綴 evolution）
    pub fn build_cli(&self, id: &str, args: &[(String, String)]) -> Option<String> {
        let cmd = self.get(id)?;
        let mut cli = format!("evolution {}", cmd.cli);
        for (slot_name, value) in args {
            // 簡單替換：cli 中的 {slot_name} → value
            cli = cli.replace(&format!("{{{}}}", slot_name), value);
        }
        Some(cli)
    }
}

impl Default for IntentCommandDB {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// 內建命令集
// ═══════════════════════════════════════════════════════════════════

fn builtin_commands() -> Vec<CommandDef> {
    vec![
        // ─── Board（狀態看板）───────────────────────────────────────
        CommandDef {
            id: "board_status".into(),
            category: "board".into(),
            aliases: vec![
                "系統狀態".into(),
                "進度表".into(),
                "看板".into(),
                "現在在做什麼".into(),
                "tasks".into(),
                "任務狀態".into(),
                "有什麼在跑".into(),
            ],
            description: "顯示所有任務的階段與進度".into(),
            cli: "board".into(),
            slots: vec![],
            examples: vec![
                "看一下系統狀態".into(),
                "現在有哪些任務在跑".into(),
                "幫我查進度".into(),
            ],
        },
        CommandDef {
            id: "board_events".into(),
            category: "board".into(),
            aliases: vec![
                "事件".into(),
                "通知".into(),
                "主動事件".into(),
                "alert".into(),
                "有新消息嗎".into(),
                "有什麼新消息".into(),
                "有什麼要告訴我的".into(),
            ],
            description: "顯示未送達的主動事件".into(),
            cli: "board events".into(),
            slots: vec![],
            examples: vec![
                "有什麼新消息".into(),
                "有什麼要告訴我的".into(),
            ],
        },
        CommandDef {
            id: "board_clear".into(),
            category: "board".into(),
            aliases: vec![
                "清除已完成".into(),
                "清空任務".into(),
                "清理看板".into(),
            ],
            description: "清除已完成任務".into(),
            cli: "board clear".into(),
            slots: vec![],
            examples: vec![
                "把完成的任務清掉".into(),
            ],
        },
        CommandDef {
            id: "board_task_detail".into(),
            category: "board".into(),
            aliases: vec![
                "任務詳情".into(),
                "查某個任務".into(),
                "任務id".into(),
            ],
            description: "顯示單一任務詳細資訊".into(),
            cli: "board {task_id}".into(),
            slots: vec![
                CommandSlot {
                    name: "task_id".into(),
                    description: "任務 UUID".into(),
                    required: true,
                },
            ],
            examples: vec![
                "任務123的詳情".into(),
            ],
        },
        // ─── Planner（任務規劃）────────────────────────────────────
        CommandDef {
            id: "planner_analyze".into(),
            category: "planner".into(),
            aliases: vec![
                "分析".into(),
                "規劃".into(),
                "幫我規劃".into(),
                "怎麼做".into(),
                "建議".into(),
            ],
            description: "分析任務複雜度並產生實作計畫".into(),
            cli: "analyze {task_description}".into(),
            slots: vec![
                CommandSlot {
                    name: "task_description".into(),
                    description: "要分析的任務描述".into(),
                    required: true,
                },
            ],
            examples: vec![
                "幫我分析這個需求".into(),
                "幫我規劃一個電商系統".into(),
                "我想做一個庫存管理系統".into(),
            ],
        },
        CommandDef {
            id: "planner_list_skills".into(),
            category: "planner".into(),
            aliases: vec![
                "技能列表".into(),
                "可用技能".into(),
                "skills".into(),
            ],
            description: "列出所有可用技能".into(),
            cli: "list-skills".into(),
            slots: vec![],
            examples: vec![
                "有哪些技能可用".into(),
            ],
        },
        // ─── System（系統操作）────────────────────────────────────
        CommandDef {
            id: "system_status".into(),
            category: "system".into(),
            aliases: vec![
                "狀態".into(),
                "環境".into(),
                "環境狀態".into(),
                "model".into(),
            ],
            description: "顯示 Ollama 模型與 Evolution 專案狀態".into(),
            cli: "status".into(),
            slots: vec![],
            examples: vec![
                "Ollama 啟動了嗎".into(),
                "現在環境怎麼樣".into(),
            ],
        },
        CommandDef {
            id: "system_check".into(),
            category: "system".into(),
            aliases: vec![
                "檢查".into(),
                "環境檢查".into(),
                "確認環境".into(),
            ],
            description: "檢查系統環境（Rust/Ollama/模型）".into(),
            cli: "boot --check-only".into(),
            slots: vec![],
            examples: vec![
                "幫我檢查一下環境".into(),
            ],
        },
        CommandDef {
            id: "system_boot".into(),
            category: "system".into(),
            aliases: vec![
                "開機".into(),
                "啟動系統".into(),
                "初始化".into(),
            ],
            description: "四階段開機流程".into(),
            cli: "boot".into(),
            slots: vec![],
            examples: vec![
                "啟動 Evolution OS".into(),
            ],
        },
        // ─── Dev（開發者）──────────────────────────────────────────
        CommandDef {
            id: "dev_new_project".into(),
            category: "dev".into(),
            aliases: vec![
                "新專案".into(),
                "建立專案".into(),
                "創建專案".into(),
            ],
            description: "建立新 Evolution 專案".into(),
            cli: "new {project_name}".into(),
            slots: vec![
                CommandSlot {
                    name: "project_name".into(),
                    description: "專案名稱".into(),
                    required: true,
                },
            ],
            examples: vec![
                "幫我建立一個電商專案".into(),
            ],
        },
        // ─── Query（查詢）─────────────────────────────────────────
        CommandDef {
            id: "query_help".into(),
            category: "query".into(),
            aliases: vec![
                "幫助".into(),
                "help".into(),
                "指令".into(),
                "可以做什麼".into(),
            ],
            description: "顯示所有可用指令".into(),
            cli: "shell --help".into(),
            slots: vec![],
            examples: vec![
                "你可以做什麼".into(),
                "幫助".into(),
            ],
        },
        CommandDef {
            id: "query_hello".into(),
            category: "query".into(),
            aliases: vec![
                "嗨".into(),
                "hi".into(),
                "你好".into(),
                "早".into(),
            ],
            description: "問候語".into(),
            cli: "shell".into(),
            slots: vec![],
            examples: vec![
                "嗨".into(),
                "你好".into(),
            ],
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════
// IntentRouter — 自然語言 → 命令路由
// ═══════════════════════════════════════════════════════════════════

/// 意圖路由結果
#[derive(Debug, Clone)]
pub struct IntentMatch {
    /// 匹配到的命令 ID
    pub command_id: String,
    /// 置信度（0.0 ~ 1.0）
    pub confidence: f32,
    /// 提取到的參數（slot_name → value）
    pub slots: HashMap<String, String>,
    /// 執行的完整 CLI 字串
    pub cli: String,
}

/// 意圖路由器
pub struct IntentRouter {
    db: IntentCommandDB,
}

impl IntentRouter {
    pub fn new() -> Self {
        Self {
            db: IntentCommandDB::new(),
        }
    }

    /// 直接關鍵字匹配（本地模型不可用時的 fallback）
    pub fn route_keyword(&self, input: &str) -> Option<IntentMatch> {
        self.db.find_best_match(input).map(|cmd| {
            let cli = format!("evolution {}", cmd.cli);
            IntentMatch {
                command_id: cmd.id.clone(),
                confidence: 0.7,
                slots: HashMap::new(),
                cli,
            }
        })
    }

    /// 使用本地模型分析意圖
    /// 實作：prompt 模型列出所有 command id + aliases，讓模型輸出最接近的
    pub fn route_with_model(
        &self,
        input: &str,
        backend: &dyn evolution_os::model::ModelDispatcher,
    ) -> Result<IntentMatch, String> {
        let command_list = self
            .db
            .commands
            .iter()
            .map(|c| format!("- {} ({})\n  aliases: {}\n  description: {}",
                c.id, c.category, c.aliases.join(", "), c.description))
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            "你是一個意圖分類器。根據用戶輸入，從以下命令列表中選擇最合適的 command_id。\n\
             只回覆 command_id，不要多餘的文字。\n\
             如果沒有完全匹配的回覆 \"none\"。\n\n\
             命令列表：\n{}\n\n\
             用戶輸入：{}\n\n\
             command_id：",
            command_list, input
        );

        let req = evolution_os::model::ModelRequest::new("gemma4:e2b", &prompt)
            .with_temperature(0.0);
        let resp = backend.dispatch(req).map_err(|e| e.to_string())?;
        let selected_id = resp.content.trim().to_lowercase();

        if selected_id == "none" || selected_id.is_empty() {
            // fallback 到關鍵字匹配
            return self
                .route_keyword(input)
                .ok_or_else(|| "無法理解輸入，請換個方式說".to_string());
        }

        let cmd = self.db.get(&selected_id)
            .ok_or_else(|| format!("模型回覆了不存在的 command_id: {}", selected_id))?;

        let cli = format!("evolution {}", cmd.cli);
        Ok(IntentMatch {
            command_id: cmd.id.clone(),
            confidence: 0.9,
            slots: HashMap::new(),
            cli,
        })
    }

    /// 取得完整的命令資料庫（用於顯示或调试）
    pub fn db(&self) -> &IntentCommandDB {
        &self.db
    }
}

impl Default for IntentRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matching() {
        let router = IntentRouter::new();

        // 測試系統狀態
        let result = router.route_keyword("看一下系統狀態").unwrap();
        assert_eq!(result.command_id, "board_status");

        // 測試事件
        let result = router.route_keyword("有什麼新消息").unwrap();
        assert_eq!(result.command_id, "board_events");

        // 測試分析
        let result = router.route_keyword("幫我規劃一個庫存系統").unwrap();
        assert_eq!(result.command_id, "planner_analyze");

        // 測試問候
        let result = router.route_keyword("嗨").unwrap();
        assert_eq!(result.command_id, "query_hello");

        // 無匹配
        let result = router.route_keyword("這個問題很奇怪");
        assert!(result.is_none());
    }

    #[test]
    fn test_command_ids() {
        let router = IntentRouter::new();
        let ids = router.db.command_ids();
        assert!(ids.contains(&"board_status"));
        assert!(ids.contains(&"planner_analyze"));
        assert!(ids.contains(&"system_boot"));
    }

    #[test]
    fn test_build_cli() {
        let router = IntentRouter::new();
        let cli = router.db.build_cli(
            "planner_analyze",
            &[("task_description".into(), "建立一個部落格系統".into())],
        );
        assert!(cli.is_some());
        assert!(cli.unwrap().contains("analyze"));
    }
}