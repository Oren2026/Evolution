//! Skill — 技能定義
//!
//! 技能是 SkillNode 的具體實作，代表可复用的功能單元。

mod registry;
pub mod filesystem;
pub mod analysis;
pub mod llm;

pub use registry::SkillRegistry;

// ===== Skill Trait =====

/// 技能trait — 所有具體技能必須實現
pub trait Skill: Send + Sync {
    /// 技能唯一識別符
    fn id(&self) -> &str;
    /// 技能名稱（可讀）
    fn name(&self) -> &str;
    /// 技能描述
    fn description(&self) -> &str;
    /// 輸入格式（JSON Schema）
    fn input_format(&self) -> &str;
    /// 輸出格式（JSON Schema）
    fn output_format(&self) -> &str;
    /// 觸發關鍵詞
    fn triggers(&self) -> Vec<&str>;
    /// 此技能依賴的其他技能 ID
    fn dependencies(&self) -> Vec<&str>;
    /// 執行技能
    fn execute(&self, input: &str) -> SkillOutput;
}

/// 技能執行結果
#[derive(Debug, Clone)]
pub struct SkillOutput {
    pub success: bool,
    pub data: String,
    pub error: Option<String>,
}

impl SkillOutput {
    pub fn ok(data: &str) -> Self {
        Self { success: true, data: data.to_string(), error: None }
    }

    pub fn err(msg: &str) -> Self {
        Self { success: false, data: String::new(), error: Some(msg.to_string()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySkill;

    impl Skill for DummySkill {
        fn id(&self) -> &str { "dummy_skill" }
        fn name(&self) -> &str { "Dummy Skill" }
        fn description(&self) -> &str { "A dummy skill for testing" }
        fn input_format(&self) -> &str { "{}" }
        fn output_format(&self) -> &str { "{}" }
        fn triggers(&self) -> Vec<&str> { vec!["dummy", "test"] }
        fn dependencies(&self) -> Vec<&str> { vec![] }
        fn execute(&self, _input: &str) -> SkillOutput { SkillOutput::ok("{}") }
    }

    #[test]
    fn test_skill_output_ok() {
        let out = SkillOutput::ok(r#"{"result": "ok"}"#);
        assert!(out.success);
        assert_eq!(out.data, r#"{"result": "ok"}"#);
        assert!(out.error.is_none());
    }

    #[test]
    fn test_skill_output_err() {
        let out = SkillOutput::err("something failed");
        assert!(!out.success);
        assert!(out.data.is_empty());
        assert_eq!(out.error, Some("something failed".to_string()));
    }
}