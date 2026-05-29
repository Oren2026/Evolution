//! ModelDispatcher — AI 系統呼叫的統一是面

use serde::{Deserialize, Serialize};
use super::DispatchError;

/// AI 模型呼叫請求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    /// 模型名稱（如 "llama3", "gemma4:2b"）
    pub model: String,
    /// 使用者 prompt
    pub prompt: String,
    /// 系統提示詞（可選）
    pub system_prompt: Option<String>,
    /// 隨機性參數（預設 0.7）
    pub temperature: f32,
    /// 最大生成 token 數（可選）
    pub max_tokens: Option<u32>,
}

impl ModelRequest {
    pub fn new(model: &str, prompt: &str) -> Self {
        Self {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system_prompt: None,
            temperature: 0.7,
            max_tokens: None,
        }
    }

    pub fn with_system_prompt(mut self, system: &str) -> Self {
        self.system_prompt = Some(system.to_string());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// AI 模型呼叫回應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// 生成內容
    pub content: String,
    /// 實際使用的模型
    pub model: String,
    /// 使用的 token 數（估算）
    pub tokens_used: u32,
}

/// AI 系統呼叫 trait — 所有 AI 後端必須實現
pub trait ModelDispatcher: Send + Sync {
    /// 發送請求到 AI 模型
    fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError>;

    /// 查詢可用模型列表
    fn available_models(&self) -> Vec<String>;

    /// 健康檢查
    fn health_check(&self) -> bool {
        !self.available_models().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_request_builder() {
        let req = ModelRequest::new("llama3", "Hello, world!")
            .with_system_prompt("You are a helpful assistant.")
            .with_temperature(0.5)
            .with_max_tokens(100);

        assert_eq!(req.model, "llama3");
        assert_eq!(req.prompt, "Hello, world!");
        assert_eq!(req.system_prompt.as_deref(), Some("You are a helpful assistant."));
        assert_eq!(req.temperature, 0.5);
        assert_eq!(req.max_tokens, Some(100));
    }

    #[test]
    fn test_model_request_defaults() {
        let req = ModelRequest::new("gemma4", "test");
        assert_eq!(req.temperature, 0.7);
        assert!(req.system_prompt.is_none());
        assert!(req.max_tokens.is_none());
    }

    #[test]
    fn test_model_response() {
        let resp = ModelResponse {
            content: "Hello!".into(),
            model: "llama3".into(),
            tokens_used: 5,
        };
        assert_eq!(resp.content, "Hello!");
        assert_eq!(resp.tokens_used, 5);
    }
}