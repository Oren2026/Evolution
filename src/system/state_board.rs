//! State Board — 系統狀態表
//!
//! 持久化的任務進度表，讓系統有「存在感」：
//! - 任務階段追蹤（Pending → Planning → Executing → Done/Blocked）
//! - 心跳機制（系統在線狀態）
//! - 主動事件佇列（系統 → 使用者的主動通知）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::model::{ModelDispatcher, ModelRequest};

// ═══════════════════════════════════════════════════════════════════
// 資料模型
// ═══════════════════════════════════════════════════════════════════

/// 任務階段
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStage {
    Pending,    // 已建立，還未開始
    Planning,   // Planner 分析中
    Executing,  // 執行中
    Done,       // 完成
    Blocked,    // 卡住（需要用戶決策）
}

impl TaskStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Planning => "Planning",
            Self::Executing => "Executing",
            Self::Done => "Done",
            Self::Blocked => "Blocked",
        }
    }
}

impl Default for TaskStage {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for TaskStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Planning => write!(f, "Planning"),
            Self::Executing => write!(f, "Executing"),
            Self::Done => write!(f, "Done"),
            Self::Blocked => write!(f, "Blocked"),
        }
    }
}

/// 階段轉換記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTransition {
    pub from: TaskStage,
    pub to: TaskStage,
    pub at: DateTime<Utc>,
    pub note: Option<String>,
}

/// 單一任務項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntry {
    pub id: String,
    pub name: String,
    pub stage: TaskStage,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub history: Vec<StageTransition>,
    pub metadata: HashMap<String, String>,
}

impl TaskEntry {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            stage: TaskStage::Pending,
            created_at: now,
            updated_at: now,
            history: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn transition_to(&mut self, new_stage: TaskStage, note: Option<String>) {
        let now = Utc::now();
        self.history.push(StageTransition {
            from: self.stage.clone(),
            to: new_stage.clone(),
            at: now,
            note,
        });
        self.stage = new_stage;
        self.updated_at = now;
    }
}

/// 系統心跳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEntry {
    pub system_name: String,
    pub last_heartbeat: DateTime<Utc>,
    pub is_alive: bool,
}

impl PresenceEntry {
    pub fn new(system_name: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            last_heartbeat: Utc::now(),
            is_alive: true,
        }
    }

    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        self.is_alive = true;
    }
}

/// 事件等級
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventLevel {
    Info,
    Warning,
    Done,
    NeedsUser,
}

impl std::fmt::Display for EventLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Done => write!(f, "DONE"),
            Self::NeedsUser => write!(f, "NEEDS_USER"),
        }
    }
}

/// 主動事件（系統 → 使用者）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub id: String,
    pub task_id: Option<String>,
    pub level: EventLevel,
    pub message: String,
    /// LLM 解讀結果（Interpretation）— 階段轉換時由 EventGenerator 生成
    pub interpretation: Option<String>,
    pub created_at: DateTime<Utc>,
    pub delivered: bool,
}

impl EventEntry {
    pub fn new(level: EventLevel, message: String, task_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_id,
            level,
            message,
            interpretation: None,
            created_at: Utc::now(),
            delivered: false,
        }
    }

    /// 使用完整的 interpretation（通常來自 EventGenerator）
    pub fn with_interpretation(mut self, interpretation: String) -> Self {
        self.interpretation = Some(interpretation);
        self
    }

    pub fn done(message: String, task_id: Option<String>) -> Self {
        Self::new(EventLevel::Done, message, task_id)
    }

    pub fn needs_user(message: String, task_id: Option<String>) -> Self {
        Self::new(EventLevel::NeedsUser, message, task_id)
    }
}

/// 完整的系統狀態快照（用於 JSON 持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBoardSnapshot {
    pub version: String,
    pub tasks: Vec<TaskEntry>,
    pub presence: PresenceEntry,
    pub events: Vec<EventEntry>,
}

impl Default for StateBoardSnapshot {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            tasks: Vec::new(),
            presence: PresenceEntry::new("EvolutionOS"),
            events: Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// StateBoard 本體
// ═══════════════════════════════════════════════════════════════════

/// 系統狀態表
#[derive(Debug, Clone)]
pub struct StateBoard {
    tasks: Vec<TaskEntry>,
    presence: PresenceEntry,
    events: Vec<EventEntry>,
}

impl StateBoard {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            presence: PresenceEntry::new("EvolutionOS"),
            events: Vec::new(),
        }
    }

    // ─── 任務管理 ────────────────────────────────────────────────

    /// 建立新任務，回傳任務 ID
    pub fn create_task(&mut self, name: String) -> String {
        let task = TaskEntry::new(name);
        let id = task.id.clone();
        self.tasks.push(task);
        id
    }

    /// 取得任務（依 ID）
    pub fn get_task(&self, id: &str) -> Option<&TaskEntry> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// 取得任務（可變，內部使用）
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut TaskEntry> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// 更新任務階段，回傳是否成功
    ///
    /// `backend` — 可選的 LLM backend，會用 EventGenerator 解讀階段轉換
    pub fn update_stage(
        &mut self,
        id: &str,
        stage: TaskStage,
        note: Option<String>,
        backend: Option<&dyn crate::model::ModelDispatcher>,
    ) -> bool {
        let task = match self.get_task_mut(id) {
            Some(t) => t,
            None => return false,
        };
        let from_stage = task.stage.clone();
        task.transition_to(stage.clone(), note);

        // 自動發出事件（帶 LLM 解讀）
        let event = match &stage {
            TaskStage::Done => {
                let base = format!("任務「{}」已完成", task.name);
                let msg = Self::llm_interpret(&task.name, &from_stage, &stage, backend)
                    .map(| interp | format!("{} — {}", base, interp))
                    .unwrap_or(base);
                Some(EventEntry::done(msg, Some(id.to_string())))
            }
            TaskStage::Blocked => {
                let base = format!("任務「{}」需要你做決定", task.name);
                let msg = Self::llm_interpret(&task.name, &from_stage, &stage, backend)
                    .map(| interp | format!("{} — {}", base, interp))
                    .unwrap_or(base);
                Some(EventEntry::needs_user(msg, Some(id.to_string())))
            }
            _ => None,
        };
        if let Some(evt) = event {
            self.events.push(evt);
        }

        true
    }

    /// 使用 EventGenerator 生成 LLM 解讀（可選）
    fn llm_interpret(
        task_name: &str,
        from: &TaskStage,
        to: &TaskStage,
        backend: Option<&dyn crate::model::ModelDispatcher>,
    ) -> Option<String> {
        use crate::system::EventGenerator;
        let gen = EventGenerator::new();
        gen.interpret_transition(task_name, from, to, backend)
    }

    /// 刪除任務（已完成或已放棄）
    pub fn remove_task(&mut self, id: &str) -> bool {
        let len_before = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        self.tasks.len() < len_before
    }

    /// 列出所有任務
    pub fn list_tasks(&self) -> &[TaskEntry] {
        &self.tasks
    }

    /// 依階段列出任務
        pub fn tasks_by_stage(&self, stage: &TaskStage) -> Vec<&TaskEntry> {
            self.tasks.iter().filter(|t| &t.stage == stage).collect()
        }

    /// 清除已完成任務
    pub fn clear_done(&mut self) -> usize {
        let before = self.tasks.len();
        self.tasks.retain(|t| t.stage != TaskStage::Done);
        before - self.tasks.len()
    }

    // ─── 心跳 ────────────────────────────────────────────────────

    /// 更新心跳
    pub fn heartbeat(&mut self) {
        self.presence.heartbeat();
    }

    /// 取得心跳狀態
    pub fn presence(&self) -> &PresenceEntry {
        &self.presence
    }

    // ─── 事件 ────────────────────────────────────────────────────

    /// 寫入自訂事件
    pub fn push_event(&mut self, event: EventEntry) {
        self.events.push(event);
    }

    /// 取得未送達事件
    pub fn undelivered_events(&self) -> Vec<&EventEntry> {
        self.events.iter().filter(|e| !e.delivered).collect()
    }

    /// 標記事件為已送達
    pub fn mark_event_delivered(&mut self, id: &str) -> bool {
        if let Some(evt) = self.events.iter_mut().find(|e| e.id == id) {
            evt.delivered = true;
            true
        } else {
            false
        }
    }

    /// 取得並清除所有未送達事件（原子操作）
    pub fn drain_undelivered(&mut self) -> Vec<EventEntry> {
        let ids: Vec<String> = self
            .events
            .iter()
            .filter(|e| !e.delivered)
            .map(|e| e.id.clone())
            .collect();
        let mut drained = Vec::new();
        for id in &ids {
            if let Some(idx) = self.events.iter().position(|e| e.id == *id) {
                self.events[idx].delivered = true;
                drained.push(self.events[idx].clone());
            }
        }
        drained
    }

    // ─── 持久化 ──────────────────────────────────────────────────

    /// 匯出快照（供 JsonStorage 序列化）
    pub fn snapshot(&self) -> StateBoardSnapshot {
        StateBoardSnapshot {
            version: env!("CARGO_PKG_VERSION").to_string(),
            tasks: self.tasks.clone(),
            presence: self.presence.clone(),
            events: self.events.clone(),
        }
    }

    /// 從快照還原（供 JsonStorage 反序列化）
    pub fn from_snapshot(snap: StateBoardSnapshot) -> Self {
        Self {
            tasks: snap.tasks,
            presence: snap.presence,
            events: snap.events,
        }
    }

    /// 產生摘要（供 CLI 顯示）
    pub fn summary(&self) -> BoardSummary {
        BoardSummary {
            total: self.tasks.len(),
            by_stage: {
                let mut m = HashMap::new();
                for t in &self.tasks {
                    *m.entry(t.stage.as_str().to_string()).or_insert(0) += 1;
                }
                m
            },
            undelivered_events: self.events.iter().filter(|e| !e.delivered).count(),
            last_heartbeat: self.presence.last_heartbeat,
            is_alive: self.presence.is_alive,
        }
    }

    // ─── LLM 主動理解 ─────────────────────────────────────────────

    /// 使用 LLM 讀取未送達事件，生成系統建議
    ///
    /// `backend` — LLM backend（Required，否則回 Err）
    ///
    /// 回傳：LLM 生成的系統建議字串，失敗時回 Err
    pub fn understand_events(
        &self,
        backend: &dyn ModelDispatcher,
    ) -> Result<String, String> {
        use crate::system::EventGenerator;

        let undelivered = self.undelivered_events();
        if undelivered.is_empty() {
            return Ok("沒有待處理事件。系統正常運行。".to_string());
        }

        // 將事件序列化為 prompt 上下文
        let events_context = undelivered
            .iter()
            .map(|e| {
                let interp = e.interpretation.as_deref().unwrap_or("(無 LLM 解讀)");
                format!(
                    "- [{}] 任務「{}」：{} — 解讀：{}",
                    e.level,
                    e.task_id.as_deref().unwrap_or("(未知)"),
                    e.message,
                    interp
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let task_summary = self
            .tasks
            .iter()
            .map(|t| format!("- 「{}」：{}", t.name, t.stage))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "以下是 Evolution OS 的系統狀態，請分析並給出系統建議：\n\n== 待處理事件 ==\n{}\n\n== 目前任務狀態 ==\n{}\n\n請用繁體中文：\n1. 判斷是否有需要使用者注意的問題\n2. 指出系統瓶頸或風險\n3. 建議下一個優先處理的方向\n\n格式：直接回覆建議，100-200 字，用自然段落。",
            events_context,
            task_summary
        );

        let gen = EventGenerator::new();
        let req = ModelRequest::new(gen.default_model(), &prompt)
            .with_temperature(0.4)
            .with_max_tokens(300);

        let resp = backend
            .dispatch(req)
            .map_err(|e| format!("LLM dispatch failed: {}", e))?;

        Ok(resp.content.trim().to_string())
    }
}

impl Default for StateBoard {
    fn default() -> Self {
        Self::new()
    }
}

/// 狀態表摘要（用於快速顯示）
#[derive(Debug, Clone, serde::Serialize)]
pub struct BoardSummary {
    pub total: usize,
    pub by_stage: HashMap<String, usize>,
    pub undelivered_events: usize,
    pub last_heartbeat: DateTime<Utc>,
    pub is_alive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_task() {
        let mut board = StateBoard::new();
        let id = board.create_task("測試任務".into());
        assert!(!id.is_empty());
        assert_eq!(board.get_task(&id).unwrap().name, "測試任務");
    }

    #[test]
    fn test_stage_transition() {
        let mut board = StateBoard::new();
        let id = board.create_task("test".into());
        assert_eq!(board.get_task(&id).unwrap().stage, TaskStage::Pending);

        board.update_stage(&id, TaskStage::Planning, None, None);
        assert_eq!(board.get_task(&id).unwrap().stage, TaskStage::Planning);

        // 歷史記錄
        let history = &board.get_task(&id).unwrap().history;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].from, TaskStage::Pending);
        assert_eq!(history[0].to, TaskStage::Planning);
    }

    #[test]
    fn test_done_generates_event() {
        let mut board = StateBoard::new();
        let id = board.create_task("my-task".into());
        board.update_stage(&id, TaskStage::Executing, None, None);
        board.update_stage(&id, TaskStage::Done, None, None);

        let events: Vec<_> = board.undelivered_events().into_iter().cloned().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, EventLevel::Done);
        assert_eq!(events[0].task_id.as_deref(), Some(id.as_str()));
    }

    #[test]
    fn test_blocked_generates_event() {
        let mut board = StateBoard::new();
        let id = board.create_task("my-task".into());
        board.update_stage(&id, TaskStage::Blocked, None, None);

        let events: Vec<_> = board.undelivered_events().into_iter().cloned().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, EventLevel::NeedsUser);
    }

    #[test]
    fn test_remove_task() {
        let mut board = StateBoard::new();
        let id = board.create_task("to-delete".into());
        assert!(board.remove_task(&id));
        assert!(board.get_task(&id).is_none());
        assert!(!board.remove_task(&id)); // 再次刪除失敗
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let mut board = StateBoard::new();
        board.create_task("t1".into());
        let id2 = board.create_task("t2".into());
        board.update_stage(&id2, TaskStage::Done, None, None);

        let snap = board.snapshot();
        let restored = StateBoard::from_snapshot(snap);

        assert_eq!(restored.tasks.len(), 2);
        assert_eq!(restored.events.len(), 1);
    }

    #[test]
    fn test_heartbeat() {
        let mut board = StateBoard::new();
        let before = board.presence().last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));
        board.heartbeat();
        assert!(board.presence().last_heartbeat > before);
    }
}