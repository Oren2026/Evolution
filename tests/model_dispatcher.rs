//! Unit Tests — ModelDispatcher
//!
//! 測試 ModelRequest、ModelResponse、ModelDispatcher trait 的行為。
//! 不依賴真實 LLM 呼叫。

use evolution_os::model::{ModelRequest, ModelResponse, ModelDispatcher, DispatchError};
use std::sync::{Arc, Mutex};

/// MockDispatcher — 模擬 ModelDispatcher 用於測試
struct MockDispatcher {
    canned_response: ModelResponse,
    call_count: Arc<Mutex<usize>>,
    models: Vec<String>,
}

impl MockDispatcher {
    fn new(canned_response: ModelResponse, models: Vec<String>) -> Self {
        Self {
            canned_response,
            call_count: Arc::new(Mutex::new(0)),
            models,
        }
    }

    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

impl ModelDispatcher for MockDispatcher {
    fn dispatch(&self, _req: ModelRequest) -> Result<ModelResponse, DispatchError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        Ok(self.canned_response.clone())
    }

    fn available_models(&self) -> Vec<String> {
        self.models.clone()
    }
}

#[test]
fn test_model_request_builder_chain() {
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
    let req = ModelRequest::new("gemma4", "test prompt");

    assert_eq!(req.temperature, 0.7);
    assert!(req.system_prompt.is_none());
    assert!(req.max_tokens.is_none());
    assert_eq!(req.model, "gemma4");
    assert_eq!(req.prompt, "test prompt");
}

#[test]
fn test_model_response_fields_accessible() {
    let resp = ModelResponse {
        content: "Hello, AI!".into(),
        model: "llama3".into(),
        tokens_used: 42,
    };

    assert_eq!(resp.content, "Hello, AI!");
    assert_eq!(resp.model, "llama3");
    assert_eq!(resp.tokens_used, 42);
}

#[test]
fn test_mock_dispatcher_tracks_call_count() {
    let canned = ModelResponse {
        content: "mock response".into(),
        model: "test-model".into(),
        tokens_used: 10,
    };
    let dispatcher = MockDispatcher::new(canned, vec!["test-model".to_string()]);

    assert_eq!(dispatcher.call_count(), 0);

    let req = ModelRequest::new("test-model", "prompt");
    let _ = dispatcher.dispatch(req.clone());
    assert_eq!(dispatcher.call_count(), 1);

    let _ = dispatcher.dispatch(req);
    assert_eq!(dispatcher.call_count(), 2);
}

#[test]
fn test_mock_dispatcher_available_models() {
    let canned = ModelResponse {
        content: "test".into(),
        model: "test-model".into(),
        tokens_used: 1,
    };
    let dispatcher = MockDispatcher::new(
        canned,
        vec!["test-model".to_string(), "other-model".to_string()],
    );

    let models = dispatcher.available_models();
    assert_eq!(models.len(), 2);
    assert!(models.contains(&"test-model".to_string()));
    assert!(models.contains(&"other-model".to_string()));
}

#[test]
fn test_health_check_true_when_models_available() {
    let canned = ModelResponse {
        content: "ok".into(),
        model: "test-model".into(),
        tokens_used: 1,
    };
    let dispatcher = MockDispatcher::new(canned, vec!["test-model".to_string()]);

    assert!(dispatcher.health_check());
}

#[test]
fn test_health_check_false_when_no_models() {
    let canned = ModelResponse {
        content: "ok".into(),
        model: "test-model".into(),
        tokens_used: 1,
    };
    let dispatcher = MockDispatcher::new(canned, vec![]);

    assert!(!dispatcher.health_check());
}

#[test]
fn test_dispatch_returns_correct_content() {
    let canned = ModelResponse {
        content: "expected content".into(),
        model: "test-model".into(),
        tokens_used: 99,
    };
    let dispatcher = MockDispatcher::new(canned, vec!["test-model".to_string()]);

    let req = ModelRequest::new("test-model", "any prompt");
    let result = dispatcher.dispatch(req).unwrap();

    assert_eq!(result.content, "expected content");
    assert_eq!(result.model, "test-model");
    assert_eq!(result.tokens_used, 99);
}

#[test]
fn test_dispatch_returns_error() {
    #[derive(Clone)]
    struct ErrorDispatcher {
        call_count: Arc<Mutex<usize>>,
    }

    impl ErrorDispatcher {
        fn new() -> Self {
            Self {
                call_count: Arc::new(Mutex::new(0)),
            }
        }
    }

    impl ModelDispatcher for ErrorDispatcher {
        fn dispatch(&self, _req: ModelRequest) -> Result<ModelResponse, DispatchError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            Err(DispatchError::ModelNotFound("test-model".to_string()))
        }

        fn available_models(&self) -> Vec<String> {
            vec![]
        }
    }

    let dispatcher = ErrorDispatcher::new();
    let req = ModelRequest::new("test-model", "prompt");
    let result = dispatcher.dispatch(req);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, DispatchError::ModelNotFound(ref m) if m == "test-model"));
}
