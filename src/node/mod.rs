//! Node — 節點抽象定義
//!
//! 所有節點的共同介面：技能、上下文、呼叫鏈都遵循同一個 trait。

mod memory_graph;
mod registry;
mod skill_node;
mod dyn_skill_node;

pub use dyn_skill_node::DynSkillNode;
pub use memory_graph::MemoryGraph;
pub use registry::{NodeHandle, NodeRegistry};
pub use skill_node::SkillNode;

use std::any::Any;

/// 節點類別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    /// 技能節點 — 可复用的功能單元
    Skill,
    /// 上下文節點 — 使用者輸入、對話狀態等
    Context,
    /// 呼叫鏈節點 — 已驗證的依賴路徑
    Chain,
}

/// 執行上下文
#[derive(Debug, Clone)]
pub struct Context {
    /// 目前正在執行的葉節點 ID
    pub leaf_id: String,
    /// 父節點棧（用於追蹤呼叫鏈）
    parent_stack: Vec<String>,
    /// 附加資料（KV pairs）
    data: std::collections::HashMap<String, String>,
}

impl Context {
    pub fn new(leaf_id: &str) -> Self {
        Self {
            leaf_id: leaf_id.to_string(),
            parent_stack: Vec::new(),
            data: std::collections::HashMap::new(),
        }
    }

    pub fn push_parent(&mut self, node_id: &str) {
        self.parent_stack.push(node_id.to_string());
    }

    pub fn get_parents(&self) -> &[String] {
        &self.parent_stack
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

/// 節點trait — 所有節點必須實現的介面
pub trait Node: Any {
    /// 節點唯一識別符
    fn id(&self) -> &str;

    /// 依賴的其他節點 ID
    fn dependencies(&self) -> Vec<&str>;

    /// 節點類別
    fn category(&self) -> NodeCategory;

    /// 執行節點邏輯
    fn execute(&self, ctx: &Context) -> NodeResult;

    /// 取得節點的具體類別資訊（用於除錯）
    fn as_any(&self) -> &dyn Any;
}

/// 節點執行結果
#[derive(Debug, Clone)]
pub struct NodeResult {
    /// 是否成功
    pub success: bool,
    /// 輸出資料（JSON字串）
    pub output: String,
    /// 錯誤訊息
    pub error: Option<String>,
}

impl NodeResult {
    pub fn ok(output: &str) -> Self {
        Self {
            success: true,
            output: output.to_string(),
            error: None,
        }
    }

    pub fn err(msg: &str) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(msg.to_string()),
        }
    }

    pub fn from_skill_output(output: crate::skill::SkillOutput) -> Self {
        if output.success {
            NodeResult::ok(&output.data)
        } else {
            NodeResult::err(output.error.as_deref().unwrap_or("unknown skill error"))
        }
    }
}

/// 呼叫鏈節點 — 記錄已驗證的依賴路徑
#[derive(Debug, Clone)]
pub struct ChainNode {
    /// 葉節點 ID
    pub leaf_id: String,
    /// 完整路徑（從葉到根）
    pub path: Vec<String>,
    /// 是否已驗證可用
    pub verified: bool,
    /// 驗證時間戳
    pub verified_at: Option<i64>,
}

impl ChainNode {
    pub fn new(leaf_id: &str, path: Vec<String>) -> Self {
        Self {
            leaf_id: leaf_id.to_string(),
            path,
            verified: false,
            verified_at: None,
        }
    }

    pub fn mark_verified(&mut self) {
        self.verified = true;
        self.verified_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Context Tests =====

    #[test]
    fn test_context_new() {
        let ctx = Context::new("leaf_node");
        assert_eq!(ctx.leaf_id, "leaf_node");
        assert!(ctx.get_parents().is_empty());
    }

    #[test]
    fn test_context_push_parent() {
        let mut ctx = Context::new("leaf");
        ctx.push_parent("parent_a");
        ctx.push_parent("parent_b");
        assert_eq!(ctx.get_parents(), &["parent_a", "parent_b"]);
    }

    #[test]
    fn test_context_data() {
        let mut ctx = Context::new("leaf");
        ctx.insert("key1", "value1");
        ctx.insert("key2", "value2");
        assert_eq!(ctx.get("key1"), Some("value1"));
        assert_eq!(ctx.get("nonexistent"), None);
    }

    // ===== NodeResult Tests =====

    #[test]
    fn test_node_result_ok() {
        let result = NodeResult::ok("output data");
        assert!(result.success);
        assert_eq!(result.output, "output data");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_node_result_err() {
        let result = NodeResult::err("something went wrong");
        assert!(!result.success);
        assert!(result.output.is_empty());
        assert_eq!(result.error, Some("something went wrong".to_string()));
    }

    // ===== ChainNode Tests =====

    #[test]
    fn test_chain_node_new() {
        let chain = ChainNode::new("leaf", vec!["leaf".to_string(), "parent".to_string(), "root".to_string()]);
        assert_eq!(chain.leaf_id, "leaf");
        assert_eq!(chain.path.len(), 3);
        assert!(!chain.verified);
        assert!(chain.verified_at.is_none());
    }

    #[test]
    fn test_chain_node_mark_verified() {
        let mut chain = ChainNode::new("leaf", vec!["leaf".to_string()]);
        assert!(!chain.verified);
        chain.mark_verified();
        assert!(chain.verified);
        assert!(chain.verified_at.is_some());
    }

    // ===== NodeCategory Tests =====

    #[test]
    fn test_node_category_equality() {
        assert_eq!(NodeCategory::Skill, NodeCategory::Skill);
        assert_eq!(NodeCategory::Context, NodeCategory::Context);
        assert_eq!(NodeCategory::Chain, NodeCategory::Chain);
        assert_ne!(NodeCategory::Skill, NodeCategory::Context);
    }
}