//! DispatchError — AI 系統呼叫的錯誤類型

use std::fmt;

/// AI 系統呼叫錯誤
#[derive(Debug, Clone)]
pub enum DispatchError {
    /// 模型不存在或無法載入
    ModelNotFound(String),
    /// 請求超時
    Timeout,
    /// 速率限制
    RateLimited,
    /// 底層錯誤（網路、解析等）
    BackendError(String),
    /// 無法連線到後端
    ConnectionError(String),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::ModelNotFound(m) => write!(f, "model not found: {}", m),
            DispatchError::Timeout => write!(f, "request timeout"),
            DispatchError::RateLimited => write!(f, "rate limited"),
            DispatchError::BackendError(s) => write!(f, "backend error: {}", s),
            DispatchError::ConnectionError(s) => write!(f, "connection error: {}", s),
        }
    }
}

impl std::error::Error for DispatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_error_display() {
        let e = DispatchError::ModelNotFound("llama3".into());
        assert!(e.to_string().contains("llama3"));
    }

    #[test]
    fn test_dispatch_error_clone() {
        let e = DispatchError::Timeout;
        let e2 = e.clone();
        assert!(matches!(e2, DispatchError::Timeout));
    }
}