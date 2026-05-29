//! Evolution Planner — 任務規劃與分工決策系統
//!
//! 核心流程：
//! Stage 1: 確認需求（收斂問題）
//! Stage 2: 分析問題（評估複雜度、預估節點）
//! Stage 3: 規劃派工（决定分工或獨立）
//! 輸出：JSON Manifest（含分工決策、optimized prompt）
//!
//! 核心理念：問題尚未確認前不進入解決階段

pub mod decision;
pub mod manifest;
pub mod stages;

pub use decision::*;
pub use manifest::*;
pub use stages::*;