//! Planner Stage 定義
//!
//! Stage 1: 確認需求（收斂問題）
//! Stage 2: 分析問題（評估複雜度、預估節點）
//! Stage 3: 規劃派工（决定分工或獨立）

use serde::{Deserialize, Serialize};

/// 規劃階段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// 尚未開始
    Init,
    /// Stage 1: 確認需求
    Confirming,
    /// Stage 2: 分析問題
    Analyzing,
    /// Stage 3: 規劃派工
    Planning,
    /// 完成
    Complete,
}

impl Default for Stage {
    fn default() -> Self {
        Stage::Init
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Init => write!(f, "Init"),
            Stage::Confirming => write!(f, "Confirming"),
            Stage::Analyzing => write!(f, "Analyzing"),
            Stage::Planning => write!(f, "Planning"),
            Stage::Complete => write!(f, "Complete"),
        }
    }
}