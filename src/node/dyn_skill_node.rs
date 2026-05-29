//! DynSkillNode — 動態技能節點（非泛型）
//!
//! 包裝 `Box<dyn Skill>` ，使其符合 `Node` trait。
//! 用於執行時期才知道具體 Skill 類型的情境。

use crate::node::{Context, Node, NodeCategory, NodeResult};
use crate::skill::{Skill, SkillOutput};
use std::any::Any;

/// 動態技能節點 — 包裝 Box<dyn Skill>
pub struct DynSkillNode {
    skill: Box<dyn Skill>,
    category: NodeCategory,
}

impl DynSkillNode {
    pub fn new(skill: Box<dyn Skill>) -> Self {
        Self {
            skill,
            category: NodeCategory::Skill,
        }
    }
}

impl Node for DynSkillNode {
    fn id(&self) -> &str {
        self.skill.id()
    }

    fn category(&self) -> NodeCategory {
        self.category
    }

    fn dependencies(&self) -> Vec<&str> {
        self.skill.dependencies()
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

impl DynSkillNode {
    pub fn name(&self) -> &str {
        self.skill.name()
    }

    pub fn as_dyn_skill(&self) -> Option<&dyn Skill> {
        Some(self.skill.as_ref())
    }
}

impl From<SkillOutput> for NodeResult {
    fn from(output: SkillOutput) -> Self {
        if output.success {
            NodeResult::ok(&output.data)
        } else {
            NodeResult::err(output.error.unwrap_or_default().as_str())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::SkillOutput;

    struct MockSkill {
        id: &'static str,
        deps: Vec<&'static str>,
    }

    impl Skill for MockSkill {
        fn id(&self) -> &str { self.id }
        fn name(&self) -> &str { "Mock" }
        fn description(&self) -> &str { "mock" }
        fn input_format(&self) -> &str { "{}" }
        fn output_format(&self) -> &str { "{}" }
        fn triggers(&self) -> Vec<&str> { vec![] }
        fn dependencies(&self) -> Vec<&str> { self.deps.clone() }
        fn execute(&self, _input: &str) -> SkillOutput { SkillOutput::ok("done") }
    }

    #[test]
    fn test_dyn_skill_node_execute() {
        let skill = Box::new(MockSkill { id: "test", deps: vec![] });
        let node = DynSkillNode::new(skill);
        let result = node.execute(&Context::new("test"));
        assert!(result.success);
        assert_eq!(result.output, "done");
    }

    #[test]
    fn test_dyn_skill_node_dependencies() {
        let skill = Box::new(MockSkill { id: "test", deps: vec!["dep_a", "dep_b"] });
        let node = DynSkillNode::new(skill);
        assert_eq!(node.dependencies(), vec!["dep_a", "dep_b"]);
    }
}