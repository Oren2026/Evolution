//! EventGenerator — LLM 驅動的系統事件解讀
//!
//! 當任務階段轉換時，使用 LLM 生成具體的事件描述，
//! 而不只是硬編碼的規則文字。
//!
//! 使用方式：
//!   let gen = EventGenerator::new();
//!   let description = gen.interpret_transition("建庫存系統", &TaskStage::Planning, &TaskStage::Executing, None);
//!   // => Some("系統規劃完成，正在啟動規劃執行流程")

use crate::model::{ModelDispatcher, ModelRequest};
use super::state_board::TaskStage;

/// 事件生成器
pub struct EventGenerator {
    default_model: String,
}

impl EventGenerator {
    pub fn new() -> Self {
        Self {
            default_model: "gemma4:e2b".to_string(),
        }
    }

    /// 取得預設模型名稱
    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    /// LLM 解讀階段轉換，生成更具體的事件描述
    ///
    /// `task_name` — 任務名稱
    /// `from_stage` — 轉換前階段
    /// `to_stage` — 轉換後階段
    /// `backend` — 可選的 LLM backend（None 時 fallback 到規則版本）
    ///
    /// 回傳：LLM 生成的事件描述，失敗時回 None（使用方 fallback）
    pub fn interpret_transition(
        &self,
        task_name: &str,
        from_stage: &TaskStage,
        to_stage: &TaskStage,
        backend: Option<&dyn ModelDispatcher>,
    ) -> Option<String> {
        let prompt = Self::build_prompt(task_name, from_stage, to_stage);

        let backend = backend?;

        let req = ModelRequest::new(&self.default_model, &prompt)
            .with_temperature(0.3)
            .with_max_tokens(120);

        let resp = backend.dispatch(req).ok()?;
        let text = resp.content.trim();

        // 解析：取第一行或第一個句子作為描述
        let description = text
            .lines()
            .next()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && l.len() >= 5)?;

        Some(description)
    }

    /// 建立解讀 prompt
    fn build_prompt(task_name: &str, from: &TaskStage, to: &TaskStage) -> String {
        let from_str = from.as_str();
        let to_str = to.as_str();

        format!(
            r#"任務「{}」的狀態從 [{}] 變成 [{}]。

請用繁體中文描述這個變化對系統和對使用者的具體意義。
格式：一個描述句，10-25 字，說明發生了什麼及影響。
例如：「系統規劃完成，準備啟動執行流程」
例如：「任務遇到需要使用者決定的環節」
例如：「系統執行中，後台正在處理多方資料同步」

直接回覆描述句，不要加標題或前後文。"#,
            task_name, from_str, to_str
        )
    }

    /// 使用指定模型（測試/不同任務用不同模型）
    pub fn interpret_transition_with_model(
        &self,
        task_name: &str,
        from_stage: &TaskStage,
        to_stage: &TaskStage,
        backend: &dyn ModelDispatcher,
        model: &str,
    ) -> Option<String> {
        let prompt = Self::build_prompt(task_name, from_stage, to_stage);

        let req = ModelRequest::new(model, &prompt)
            .with_temperature(0.3)
            .with_max_tokens(120);

        let resp = backend.dispatch(req).ok()?;
        let text = resp.content.trim();

        text.lines()
            .next()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && l.len() >= 5)
    }
}

impl Default for EventGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let prompt = EventGenerator::build_prompt(
            "建庫存系統",
            &TaskStage::Planning,
            &TaskStage::Executing,
        );
        assert!(prompt.contains("建庫存系統"));
        assert!(prompt.contains("Planning"));
        assert!(prompt.contains("Executing"));
    }
}