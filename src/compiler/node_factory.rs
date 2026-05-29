//! Node Factory — 根據 EstimatedNode 建立實際可執行的節點
//!
//! 每個 EstimatedNode 代表一個角色（前端/後端/資料庫工程師），
//! NodeFactory 為每個角色建立一個 LLMNode，透過 Ollama 生成實作內容。

use crate::model::{ModelDispatcher, ModelRequest};
use crate::planner::manifest::EstimatedNode;
use crate::skill::{Skill, SkillOutput};
use std::sync::{Arc, Mutex};

/// 角色對應的 system prompt 前綴
fn role_system_prompt(role: &str, task: &str) -> String {
    match role.to_lowercase().as_str() {
        r if r.contains("前端") || r.contains("frontend") || r.contains("ui") => {
            format!(
                "你是前端工程師。根據以下任務描述，生成前端實作。\n\n\
                 任務：{}\n\n\
                 輸出格式：JSON {{ \"files\": [{{\"path\": \"...\", \"content\": \"...\"}}] }}\n\
                 只輸出符合任務的檔案，不要多餘內容。",
                task
            )
        }
        r if r.contains("後端") || r.contains("backend") || r.contains("server") => {
            format!(
                "你是後端工程師。根據以下任務描述，生成後端實作。\n\n\
                 任務：{}\n\n\
                 輸出格式：JSON {{ \"files\": [{{\"path\": \"...\", \"content\": \"...\"}}] }}\n\
                 只輸出符合任務的檔案，不要多餘內容。",
                task
            )
        }
        r if r.contains("資料庫") || r.contains("database") || r.contains("db") => {
            format!(
                "你是資料庫工程師。根據以下任務描述，生成資料庫相關實作。\n\n\
                 任務：{}\n\n\
                 輸出格式：JSON {{ \"files\": [{{\"path\": \"...\", \"content\": \"...\"}}] }}\n\
                 只輸出符合任務的檔案，不要多餘內容。",
                task
            )
        }
        r if r.contains("安全") || r.contains("auth") || r.contains("認證") => {
            format!(
                "你是安全工程師。根據以下任務描述，生成認證/安全相關實作。\n\n\
                 任務：{}\n\n\
                 輸出格式：JSON {{ \"files\": [{{\"path\": \"...\", \"content\": \"...\"}}] }}\n\
                 只輸出符合任務的檔案，不要多餘內容。",
                task
            )
        }
        _ => {
            format!(
                "你是軟體工程師。根據以下任務描述，生成實作。\n\n\
                 任務：{}\n\n\
                 輸出格式：JSON {{ \"files\": [{{\"path\": \"...\", \"content\": \"...\"}}] }}\n\
                 只輸出符合任務的檔案，不要多餘內容。",
                task
            )
        }
    }
}

/// LLMNode — Planner 規劃出的角色節點
///
/// 每個 EstimatedNode 對應一個 LLMNode，執行時透過 LLM 生成該角色的實作內容。
pub struct LLMNode {
    pub id: String,
    pub role: String,
    pub handles: Vec<String>,
    pub depends_on: Vec<String>,
    /// 共享的 model dispatcher（Arc + Mutex 支援跨執行緒）
    backend: Arc<Mutex<Box<dyn ModelDispatcher>>>,
    /// 預設模型
    model: String,
    /// 原始任務描述
    task: String,
    /// 執行結果快取（避免重複 LLM 呼叫）
    cached_output: Mutex<Option<String>>,
}

impl LLMNode {
    pub fn new(
        est: &EstimatedNode,
        backend: Arc<Mutex<Box<dyn ModelDispatcher>>>,
        task: &str,
    ) -> Self {
        Self {
            id: est.id.clone(),
            role: est.role.clone(),
            handles: est.handles.clone(),
            depends_on: est.depends_on.clone(),
            backend,
            model: "gemma4:e4b".to_string(),
            task: task.to_string(),
            cached_output: Mutex::new(None),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    fn build_request(&self, context: &str) -> ModelRequest {
        let system = role_system_prompt(&self.role, &self.task);
        let user = if context.is_empty() {
            format!("請根據上述任務生成實作。")
        } else {
            format!(
                "上游節點輸出：\n{}\n\n請根據上游輸出，生成屬於你这个角色的實作。",
                context
            )
        };

        ModelRequest::new(&self.model, &user)
            .with_system_prompt(&system)
            .with_temperature(0.3)
            .with_max_tokens(2048)
    }

    /// 檢查是否已有快取結果
    pub fn is_cached(&self) -> bool {
        self.cached_output.lock().unwrap().is_some()
    }

    /// 取得快取的輸出
    pub fn get_cached(&self) -> Option<String> {
        self.cached_output.lock().unwrap().clone()
    }
}

impl Skill for LLMNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.role
    }

    fn description(&self) -> &str {
        &self.role
    }

    fn input_format(&self) -> &str {
        "{}"
    }

    fn output_format(&self) -> &str {
        r#"{"files": [{"path": "...", "content": "..."}]}"#
    }

    fn triggers(&self) -> Vec<&str> {
        vec![]
    }

    fn dependencies(&self) -> Vec<&str> {
        self.depends_on.iter().map(|s| s.as_str()).collect()
    }

    fn execute(&self, input: &str) -> SkillOutput {
        // 如果已有快取，直接回傳
        if let Some(cached) = self.get_cached() {
            return SkillOutput::ok(&cached);
        }

        // 建 LLM 请求
        let req = self.build_request(input);

        // 發送請求
        let backend = self.backend.lock().unwrap();
        match backend.dispatch(req) {
            Ok(resp) => {
                let out = resp.content.clone();
                // 快取結果
                *self.cached_output.lock().unwrap() = Some(out.clone());
                SkillOutput::ok(&out)
            }
            Err(e) => SkillOutput::err(&format!("LLM dispatch error: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_system_prompt_frontend() {
        let prompt = role_system_prompt("前端工程師", "寫一個按鈕");
        assert!(prompt.contains("前端工程師"));
        assert!(prompt.contains("寫一個按鈕"));
    }

    #[test]
    fn test_role_system_prompt_backend() {
        let prompt = role_system_prompt("backend", "實作 REST API");
        assert!(prompt.contains("後端工程師"));
    }

    #[test]
    fn test_role_system_prompt_unknown() {
        let prompt = role_system_prompt("QA工程師", "寫測試");
        assert!(prompt.contains("軟體工程師"));
    }

    #[test]
    fn test_llm_node_caching() {
        // 這個測試驗證快取機制，不真的呼叫 LLM
        // 透過 mock backend 驗證
        let node = LLMNode {
            id: "test-node".to_string(),
            role: "前端工程師".to_string(),
            handles: vec![],
            depends_on: vec![],
            backend: Arc::new(Mutex::new(
                Box::new(crate::model::OllamaBackend::new()) as Box<dyn ModelDispatcher>
            )),
            model: "gemma4:e4b".to_string(),
            task: "test task".to_string(),
            cached_output: Mutex::new(Some("cached result".to_string())),
        };

        assert!(node.is_cached());
        assert_eq!(node.get_cached(), Some("cached result".to_string()));
    }
}
