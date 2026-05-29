//! Skill Registry — 技能注册表
//!
//! 管理所有可用技能，提供關鍵詞匹配查詢。

use super::Skill;
use std::collections::HashMap;

/// 技能注册表
pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
    keyword_index: HashMap<String, Vec<String>>, // 關鍵詞 → 技能ID列表
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            keyword_index: HashMap::new(),
        }
    }

    /// 注册技能
    pub fn register<S: Skill + 'static>(&mut self, skill: S) {
        let id = skill.id().to_string();

        // 維護關鍵詞索引
        for kw in skill.triggers() {
            self.keyword_index
                .entry(kw.to_lowercase())
                .or_default()
                .push(id.clone());
        }

        self.skills.insert(id, Box::new(skill));
    }

    /// 根據 ID 取得技能
    pub fn get(&self, id: &str) -> Option<&dyn Skill> {
        self.skills.get(id).map(|s| s.as_ref())
    }

    /// 根據關鍵詞查詢技能
    pub fn find_by_keyword(&self, keyword: &str) -> Vec<&dyn Skill> {
        let kw_lower = keyword.to_lowercase();
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        if let Some(ids) = self.keyword_index.get(&kw_lower) {
            for id in ids {
                if seen.insert(id.clone()) {
                    if let Some(skill) = self.skills.get(id) {
                        result.push(skill.as_ref());
                    }
                }
            }
        }

        result
    }

    /// 所有技能 ID
    pub fn list_ids(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Skill;

    struct TestSkill {
        id: &'static str,
        triggers_list: Vec<&'static str>,
    }

    impl Skill for TestSkill {
        fn id(&self) -> &str { self.id }
        fn name(&self) -> &str { "Test Skill" }
        fn description(&self) -> &str { "A test skill" }
        fn input_format(&self) -> &str { "{}" }
        fn output_format(&self) -> &str { "{}" }
        fn triggers(&self) -> Vec<&str> { self.triggers_list.clone() }
        fn dependencies(&self) -> Vec<&str> { vec![] }
        fn execute(&self, _input: &str) -> super::super::SkillOutput { super::super::SkillOutput::ok("{}") }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = SkillRegistry::new();
        reg.register(TestSkill { id: "skill_1", triggers_list: vec!["html", "web"] });

        assert!(reg.get("skill_1").is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_find_by_keyword() {
        let mut reg = SkillRegistry::new();
        reg.register(TestSkill { id: "html_gen", triggers_list: vec!["html", "page"] });
        reg.register(TestSkill { id: "css_gen", triggers_list: vec!["css", "style"] });

        let results = reg.find_by_keyword("html");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id(), "html_gen");
    }

    #[test]
    fn test_find_by_keyword_case_insensitive() {
        let mut reg = SkillRegistry::new();
        reg.register(TestSkill { id: "s1", triggers_list: vec!["HTML"] });

        let results = reg.find_by_keyword("html");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_ids() {
        let mut reg = SkillRegistry::new();
        reg.register(TestSkill { id: "a", triggers_list: vec![] });
        reg.register(TestSkill { id: "b", triggers_list: vec![] });

        let ids = reg.list_ids();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }
}