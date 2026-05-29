//! Model — AI 系統呼叫介面
//!
//! 將 LLM 呼叫包裝成「系統呼叫」介面：
//! - ModelDispatcher：統一的 AI 呼叫 trait
//! - ModelRequest / ModelResponse：請求與回應結構
//! - DispatchError：錯誤類型
//! - OllamaBackend：本地 Ollama 實作

mod dispatcher;
mod backend;
mod error;

pub use dispatcher::{ModelDispatcher, ModelRequest, ModelResponse};
pub use backend::OllamaBackend;
pub use error::DispatchError;