//! LLM Summarizer Skill — 展示 ModelDispatcher 整合
//!
//! 技能收到結構化資料（dirs、files、stats），呼叫 Ollama 生成自然語言摘要。

use crate::model::{DispatchError, ModelDispatcher, ModelRequest, ModelResponse};
use crate::skill::{Skill, SkillOutput};

pub struct LLMSummarizerSkill {
    backend: Box<dyn ModelDispatcher>,
    default_model: String,
}

impl LLMSummarizerSkill {
    pub fn new(backend: Box<dyn ModelDispatcher>) -> Self {
        Self {
            backend,
            default_model: "gemma4:e4b".to_string(),
        }
    }

    pub fn with_model(backend: Box<dyn ModelDispatcher>, model: &str) -> Self {
        Self {
            backend,
            default_model: model.to_string(),
        }
    }

    fn build_prompt(&self, input: &str) -> String {
        // 嘗試從 input 解析結構化資料
        let json = serde_json::from_str::<serde_json::Value>(input).ok();
        let (dirs, files, total_lines, _total_files) = json
            .map(|j| {
                let dirs = j
                    .get("dirs")
                    .and_then(|d| d.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);
                let files = j
                    .get("files")
                    .and_then(|f| f.as_array())
                    .map(|arr| arr.len())
                    .unwrap_or(0);
                let total_lines = j.get("total_lines").and_then(|l| l.as_u64()).unwrap_or(0);
                let total_files = j.get("total_files").and_then(|f| f.as_u64()).unwrap_or(0);
                (dirs, files, total_lines, total_files)
            })
            .unwrap_or((0, 0, 0, 0));

        format!(
            "You are a project analyst. Summarize this project in 2-3 sentences.\n\n\
             Project stats: {} directories, {} files, {} lines of code.\n\
             Provide a concise summary including:\n\
             - What the project does\n\
             - Programming language(s)\n\
             - Key modules or components\n\n\
             Be brief and technical.",
            dirs, files, total_lines
        )
    }
}

impl Skill for LLMSummarizerSkill {
    fn id(&self) -> &str {
        "llm.summarize"
    }

    fn name(&self) -> &str {
        "LLM Summarizer"
    }

    fn description(&self) -> &str {
        "Use Ollama LLM to generate natural language project summaries"
    }

    fn input_format(&self) -> &str {
        r#"{"dirs": [...], "files": [...], "total_lines": N, "total_files": N}"#
    }

    fn output_format(&self) -> &str {
        r#"{"summary": "natural language text"}"#
    }

    fn triggers(&self) -> Vec<&str> {
        vec!["summarize", "analyze", "report", "describe"]
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["filesystem.file_stats"]
    }

    fn execute(&self, input: &str) -> SkillOutput {
        let prompt = self.build_prompt(input);

        let req = ModelRequest::new(&self.default_model, &prompt);
        match self.backend.dispatch(req) {
            Ok(resp) => {
                let result = serde_json::json!({
                    "summary": resp.content,
                    "model": resp.model,
                    "tokens_used": resp.tokens_used,
                });
                SkillOutput::ok(&result.to_string())
            }
            Err(e) => SkillOutput::err(&format!("LLM dispatch failed: {:?}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend;

    impl ModelDispatcher for MockBackend {
        fn dispatch(&self, req: ModelRequest) -> Result<ModelResponse, DispatchError> {
            Ok(ModelResponse {
                content: format!("This is a Rust project with {} modules.", req.prompt.len()),
                model: req.model,
                tokens_used: 42,
            })
        }

        fn available_models(&self) -> Vec<String> {
            vec!["mock-model".into()]
        }
    }

    #[test]
    fn test_llm_summarizer_execute() {
        let backend = MockBackend;
        let skill = LLMSummarizerSkill::new(Box::new(backend));

        let input = r#"{"dirs": [{"name": "src"}, {"name": "tests"}], "files": [], "total_lines": 500, "total_files": 10}"#;
        let output = skill.execute(input);

        assert!(output.success, "expected success, got: {:?}", output.error);
        assert!(output.data.contains("Rust project"));
    }

    #[test]
    fn test_build_prompt_with_invalid_json() {
        let backend = MockBackend;
        let skill = LLMSummarizerSkill::new(Box::new(backend));

        let input = "not json at all";
        let prompt = skill.build_prompt(input);

        assert!(prompt.contains("0 directories"));
    }
}