//! SystemProcess — 行程包裝 Trait
//!
//! 將 Node（SkillNode、DynSkillNode）包裝成可經由 Kernel SysCall 管理的行程。
//!
//! ## 設計
//! - SystemProcess trait：所有可被 Kernel 管理的行程必須實作
//! - NodeProcess：將 Node 實例包裝為行程，透過 Kernel.syscall() 與其他行程溝通
//! - Planner/Compiler/Executor 各自是獨立的行程，透过 Kernel 協調

use crate::kernel::{Kernel, Pid, ProcessState, SysCall, SysCallKind, SysCallResult};

/// 系統行程 Trait — 所有需要由 Kernel 管理的單元都要實作這個 trait
pub trait SystemProcess {
    /// 行程名稱
    fn name(&self) -> &str;
    /// 系統提示詞（用於 LLM 決策）
    fn system_prompt(&self) -> &str;
    /// 處理訊息（Planner/Executor 發來的任務描述）
    fn handle(&mut self, message: &str, kernel: &mut Kernel) -> Result<String, String>;
    /// 是否已完成
    fn is_done(&self) -> bool;
}

/// Node 包裝為 SystemProcess
pub struct NodeProcess {
    pub pid: Pid,
    pub name: String,
    pub system_prompt: String,
    pub node_id: String,
    result: Option<String>,
}

impl NodeProcess {
    pub fn new(pid: Pid, name: &str, system_prompt: &str, node_id: &str) -> Self {
        Self {
            pid,
            name: name.to_string(),
            system_prompt: system_prompt.to_string(),
            node_id: node_id.to_string(),
            result: None,
        }
    }

    pub fn set_result(&mut self, r: String) {
        self.result = Some(r);
    }

    pub fn get_result(&self) -> Option<&String> {
        self.result.as_ref()
    }
}

impl SystemProcess for NodeProcess {
    fn name(&self) -> &str {
        &self.name
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn handle(&mut self, message: &str, _kernel: &mut Kernel) -> Result<String, String> {
        // NodeProcess 的 handle：直接處理任務（Planner/Compiler 已做决策，這裡執行）
        // 真實場景：會呼叫 node.execute()
        self.result = Some(format!("processed: {}", message));
        Ok(self.result.clone().unwrap())
    }

    fn is_done(&self) -> bool {
        self.result.is_some()
    }
}

/// Planner 行程
pub struct PlannerProcess {
    pub pid: Pid,
    pub name: String,
    pub system_prompt: String,
    manifest: Option<crate::planner::Manifest>,
}

impl PlannerProcess {
    pub fn new(pid: Pid, system_prompt: &str) -> Self {
        Self {
            pid,
            name: "planner".to_string(),
            system_prompt: system_prompt.to_string(),
            manifest: None,
        }
    }

    pub fn plan(&mut self, task: &str) -> crate::planner::Manifest {
        let manifest = crate::planner::Manifest::from_task(task);
        self.manifest = Some(manifest.clone());
        manifest
    }
}

impl SystemProcess for PlannerProcess {
    fn name(&self) -> &str {
        &self.name
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn handle(&mut self, message: &str, _kernel: &mut Kernel) -> Result<String, String> {
        let manifest = self.plan(message);
        Ok(serde_json::to_string(&manifest).unwrap_or_default())
    }

    fn is_done(&self) -> bool {
        self.manifest.is_some()
    }
}