//! OllamaBackend — 本地 Ollama 的 ModelDispatcher 實作
//!
//! 連接 `http://localhost:11434`，使用 `/api/generate` endpoint。

use super::{DispatchError, ModelDispatcher, ModelRequest, ModelResponse};

/// Ollama 後端實作
pub struct OllamaBackend {
    base_url: String,
    timeout_secs: u64,
    client: reqwest::blocking::Client,
}

impl OllamaBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_url(url: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self {
            base_url: url.trim_end_matches('/').to_string(),
            timeout_secs: 60,
            client,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    fn send_request(&self, req: &ModelRequest) -> Result<String, DispatchError> {
        let mut body = serde_json::json!({
            "model": req.model,
            "prompt": req.prompt,
            "stream": false,
            "options": {
                "temperature": req.temperature,
            }
        });

        if let Some(ref system) = req.system_prompt {
            body["system"] = serde_json::json!(system);
        }
        if let Some(tokens) = req.max_tokens {
            body["options"]["num_predict"] = serde_json::json!(tokens);
        }

        let url = format!("{}/api/generate", self.base_url);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| {
                if e.is_connect() {
                    DispatchError::ConnectionError(e.to_string())
                } else if e.is_timeout() {
                    DispatchError::Timeout
                } else {
                    DispatchError::BackendError(e.to_string())
                }
            })?;

        let json: serde_json::Value =
            resp.json()
                .map_err(|e| DispatchError::BackendError(format!("failed to parse response: {}", e)))?;

        if let Some(err_msg) = json.get("error").and_then(|e| e.as_str()) {
            if err_msg.contains("model not found") || err_msg.contains("no such file") {
                return Err(DispatchError::ModelNotFound(req.model.clone()));
            }
            return Err(DispatchError::BackendError(err_msg.to_string()));
        }

        let content = json
            .get("response")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        let model = json
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or(&req.model)
            .to_string();

        let tokens_used = json
            .get("eval_count")
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as u32;

        Ok(serde_json::to_string(&serde_json::json!({
            "content": content,
            "model": model,
            "tokens_used": tokens_used,
        })).unwrap())
    }
}

impl Default for OllamaBackend {
    fn default() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_secs: 60,
            client,
        }
    }
}

impl ModelDispatcher for OllamaBackend {
    fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError> {
        let json_str = self.send_request(&req)?;
        let resp: ModelResponse =
            serde_json::from_str(&json_str).map_err(|e| {
                DispatchError::BackendError(format!("failed to parse response struct: {}", e))
            })?;
        Ok(resp)
    }

    fn available_models(&self) -> Vec<String> {
        let url = format!("{}/api/tags", self.base_url);
        let Ok(resp) = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
        else {
            return vec![];
        };
        let Ok(json) = resp.json::<serde_json::Value>() else {
            return vec![];
        };
        json.get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockOllama;

    impl ModelDispatcher for MockOllama {
        fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError> {
            Ok(ModelResponse {
                content: format!("Echo: {}", req.prompt),
                model: req.model,
                tokens_used: req.prompt.len() as u32 / 4,
            })
        }

        fn available_models(&self) -> Vec<String> {
            vec!["llama3".into(), "gemma4:2b".into(), "mistral:7b".into()]
        }
    }

    #[test]
    fn test_mock_ollama_dispatch() {
        let backend = MockOllama;
        let req = ModelRequest::new("llama3", "Hello!");
        let resp = backend.dispatch(req).unwrap();
        assert!(resp.content.contains("Hello!"));
    }

    #[test]
    fn test_mock_available_models() {
        let backend = MockOllama;
        let models = backend.available_models();
        assert!(models.contains(&"llama3".into()));
    }

    #[test]
    fn test_ollama_backend_default_url() {
        let backend = OllamaBackend::new();
        assert_eq!(backend.available_models().len(), 0); // 無法連線時回空vec，不 error
    }
}
