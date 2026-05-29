//! Evolution OS — AI 作業系統核心
//!
//! 核心概念：
//! - **節點（Node）**：基本執行單位，分為 Skill / Context / Chain 三類
//! - **記憶圖（Memory Graph）**：持久化儲存節點和已驗證的呼叫鏈
//! - **呼叫鏈追蹤（Chain Discovery）**：從葉往上探索，依賴 `use` 宣告追蹤實際呼叫關係
//!
//! 軟體本身是一棵樹，我們透過葉來回推樹的樣貌——
//! 每次都是葉節點提出需求，往上追溯父節點的真正需求，
//! 而不是建立好一套固定環境讓我們困在版本中。

pub mod node;
pub mod chain;
pub mod skill;
pub mod runtime;
pub mod model;
pub mod storage;
pub mod evo;
pub mod planner;
pub mod compiler;
pub mod kernel;
pub mod system;

// ===== 公開主要類型 =====

pub use node::{
    Context, Node, NodeCategory, NodeResult, ChainNode,
    MemoryGraph, NodeRegistry, NodeHandle, DynSkillNode, SkillNode,
};
pub use chain::{ChainDiscovery, DiscoveryResult};
pub use skill::{SkillRegistry, Skill};

// ===== 版本標記 =====

pub const VERSION: &str = env!("CARGO_PKG_VERSION");