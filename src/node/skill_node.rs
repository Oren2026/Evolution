//! SkillNode — 包裝 Skill 使其符合 Node trait
//!
//! SkillNode 是 Node 和 Skill 中間的橋樑：
//! - Skill 定义了「如何執行」這個技能
//! - SkillNode 包裝後，使其符合 Node trait，可以加入 MemoryGraph
//! - Node 的 execute() 會調用 Skill 的 execute()，並處理 Context 轉換

use crate::node::{Context, Node, NodeCategory, NodeResult};
use crate::skill::Skill;
use std::any::Any;

/// SkillNode — 包裝 Skill 使其成為 Node
pub struct SkillNode<S: Skill> {
    skill: S,
    id: String,
}

impl<S: Skill> SkillNode<S> {
    pub fn new(skill: S) -> Self {
        let id = skill.id().to_string();
        Self { skill, id }
    }

    pub fn skill_id(&self) -> &str {
        &self.id
    }
}

impl<S: Skill + 'static> Node for SkillNode<S> {
    fn id(&self) -> &str {
        &self.id
    }

    fn dependencies(&self) -> Vec<&str> {
        self.skill.dependencies()
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Skill
    }

    fn execute(&self, ctx: &Context) -> NodeResult {
        let input = ctx.get("input").unwrap_or("{}");
        let output = self.skill.execute(input);
        NodeResult::from_skill_output(output)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{Skill, SkillOutput};
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockSkill {
        id: &'static str,
        deps: Vec<&'static str>,
        called: AtomicBool,
        output: SkillOutput,
    }

    impl MockSkill {
        fn new_ok(id: &'static str, deps: Vec<&'static str>, called: AtomicBool) -> Self {
            Self { id, deps, called, output: SkillOutput::ok("executed") }
        }
        fn new_err(id: &'static str, deps: Vec<&'static str>, msg: &str, called: AtomicBool) -> Self {
            Self { id, deps, called, output: SkillOutput::err(msg) }
        }
    }

    impl Skill for MockSkill {
        fn id(&self) -> &str { self.id }
        fn name(&self) -> &str { "Mock Skill" }
        fn description(&self) -> &str { "A mock skill for testing" }
        fn input_format(&self) -> &str { "{}" }
        fn output_format(&self) -> &str { "{}" }
        fn triggers(&self) -> Vec<&str> { vec![] }
        fn dependencies(&self) -> Vec<&str> { self.deps.clone() }
        fn execute(&self, _input: &str) -> SkillOutput {
            self.called.store(true, Ordering::SeqCst);
            self.output.clone()
        }
    }

    #[test]
    fn test_skill_node_id() {
        let called = AtomicBool::new(false);
        let node = SkillNode::new(MockSkill::new_ok("test.skill", vec![], called));
        assert_eq!(node.id(), "test.skill");
    }

    #[test]
    fn test_skill_node_dependencies() {
        let called = AtomicBool::new(false);
        let node = SkillNode::new(MockSkill::new_ok("test.skill", vec!["dep.a", "dep.b"], called));
        assert_eq!(node.dependencies(), vec!["dep.a", "dep.b"]);
    }

    #[test]
    fn test_skill_node_category() {
        let called = AtomicBool::new(false);
        let node = SkillNode::new(MockSkill::new_ok("test.skill", vec![], called));
        assert_eq!(node.category(), NodeCategory::Skill);
    }

    #[test]
    fn test_skill_node_execute_success() {
        let node = SkillNode::new(MockSkill::new_ok("test.skill", vec![], AtomicBool::new(false)));
        let result = node.execute(&Context::new("test"));
        assert!(result.success);
    }

    #[test]
    fn test_skill_node_execute_failure() {
        let node = SkillNode::new(MockSkill::new_err("test.skill", vec![], "skill error", AtomicBool::new(false)));
        let result = node.execute(&Context::new("test"));
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "skill error");
    }

    #[test]
    fn test_skill_node_with_input() {
        let called = AtomicBool::new(false);
        let node = SkillNode::new(MockSkill::new_ok("test.skill", vec![], called));
        let mut ctx = Context::new("test");
        ctx.insert("input", "{\"key\":\"value\"}");
        let result = node.execute(&ctx);
        assert!(result.success);
    }

    #[test]
    fn test_skill_node_multiple_skills() {
        let called_a = AtomicBool::new(false);
        let called_b = AtomicBool::new(false);
        let node_a = SkillNode::new(MockSkill::new_ok("skill.a", vec![], called_a));
        let node_b = SkillNode::new(MockSkill::new_ok("skill.b", vec!["skill.a"], called_b));
        assert_eq!(node_a.id(), "skill.a");
        assert_eq!(node_b.id(), "skill.b");
        assert_eq!(node_b.dependencies(), vec!["skill.a"]);
    }
}