//! SysCall — 系統呼叫
//!
//! Evolution OS 的 SysCall 是行程與 Kernel 溝通的唯一介面。
//!
//! 現有 SysCall：
//! - Spawn    — 建立新行程（包裝 Node/Skill）
//! - Send     — 傳訊息給某 PID
//! - Receive  — 從自己的信箱取訊息（blocking）
//! - Wait     — 等待某 PID 完成
//! - Exit     — 行程結束

use crate::kernel::Pid;
use crate::kernel::process::ProcessState;

/// 系統呼叫類型
#[derive(Debug, Clone)]
pub enum SysCallKind {
    /// 建立新行程（參數：名稱、系統提示詞）
    Spawn { name: String, system_prompt: String },
    /// 傳送訊息（參數：目標 PID、內容）
    Send { target: Pid, content: String },
    /// 接收訊息（blocking，直到收到訊息）
    Receive,
    /// 等待行程完成（參數：目標 PID）
    Wait { target: Pid },
    /// 行程結束（參數：exit code）
    Exit { code: i32 },
}

/// 系統呼叫封包
#[derive(Debug, Clone)]
pub struct SysCall {
    /// 發起者 PID
    pub caller: Pid,
    /// 呼叫類型
    pub kind: SysCallKind,
}

/// 系統呼叫結果
#[derive(Debug, Clone)]
pub struct SysCallResult {
    /// 是否成功
    pub ok: bool,
    /// 回傳值（視 SysCall 類型而異）
    pub value: ResultValue,
    /// 錯誤訊息
    pub error: Option<String>,
}

impl SysCallResult {
    /// 成功結果
    pub fn ok(value: ResultValue) -> Self {
        Self {
            ok: true,
            value,
            error: None,
        }
    }

    /// 失敗結果
    pub fn err(msg: &str) -> Self {
        Self {
            ok: false,
            value: ResultValue::None,
            error: Some(msg.to_string()),
        }
    }
}

/// 回傳值型別
#[derive(Debug, Clone)]
pub enum ResultValue {
    /// 無回傳值
    None,
    /// 新行程 PID（Spawn）
    Pid(Pid),
    /// 訊息內容（Receive）
    Message(String),
    /// 已完成（Wait）
    Done(Pid),
    /// Exit code（Exit）
    ExitCode(i32),
}

impl SysCall {
    /// Spawn a new process
    pub fn spawn(name: &str, system_prompt: &str, caller: Pid) -> Self {
        SysCall {
            caller,
            kind: SysCallKind::Spawn {
                name: name.to_string(),
                system_prompt: system_prompt.to_string(),
            },
        }
    }

    /// Send a message to a process
    pub fn send(target: Pid, content: String, caller: Pid) -> Self {
        SysCall {
            caller,
            kind: SysCallKind::Send { target, content },
        }
    }

    /// Receive a message (blocking)
    pub fn receive(caller: Pid, _target: Pid) -> Self {
        // Note: target is ignored in this simplified sync version
        SysCall {
            caller,
            kind: SysCallKind::Receive,
        }
    }

    /// Wait for a process to finish
    pub fn wait(target: Pid, caller: Pid) -> Self {
        SysCall {
            caller,
            kind: SysCallKind::Wait { target },
        }
    }

    /// Exit the current process
    pub fn exit(caller: Pid, code: i32) -> Self {
        SysCall {
            caller,
            kind: SysCallKind::Exit { code },
        }
    }
}

impl SysCallResult {
    /// Get the PID from the result (for Spawn)
    pub fn expect_pid(&self) -> Pid {
        match &self.value {
            ResultValue::Pid(pid) => *pid,
            _ => Pid::default(),
        }
    }
}

impl std::fmt::Display for SysCallResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ok {
            write!(f, "SysCallResult::OK({:?})", self.value)
        } else {
            write!(f, "SysCallResult::Err({})", self.error.as_deref().unwrap_or("?"))
        }
    }
}