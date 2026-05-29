//! Process — 行程
//!
//! 每個 Node 都被包裝成 Process，擁有：
//! - 唯一 PID
//! - 行程狀態（Ready / Running / Blocked / Done）
//! - 信箱（Mailbox）用於接收訊息
//! - 系統提示詞（由 NodeFactory 生成）

use crate::kernel::mailbox::Mailbox;

/// 行程 ID（0 = 無效）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid(pub usize);

impl Pid {
    pub fn new(id: usize) -> Self { Pid(id) }
    pub fn value(&self) -> usize { self.0 }
}

impl Default for Pid {
    fn default() -> Self { Pid(0) }
}

impl std::fmt::Display for Pid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PID({})", self.0)
    }
}

/// 行程狀態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// 等待被調度
    Ready,
    /// 目前正在執行
    Running,
    /// 等待某個訊息（blocking receive）
    Waiting,
    /// 執行完成
    Done,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessState::Ready => write!(f, "Ready"),
            ProcessState::Running => write!(f, "Running"),
            ProcessState::Waiting => write!(f, "Waiting"),
            ProcessState::Done => write!(f, "Done"),
        }
    }
}

/// 行程 — 最小可調度單位
pub struct Process {
    /// 唯一識別符
    pub pid: Pid,
    /// 行程名稱（通常是 node id）
    pub name: String,
    /// 目前狀態
    pub state: ProcessState,
    /// 信箱
    pub mailbox: Mailbox,
    /// 依賴的 PID（Blocking on）
    pub waiting_on: Option<Pid>,
    /// 系統提示詞
    pub system_prompt: String,
    /// 實際資料（Node、Skill 或其他）
    pub data: Option<Box<dyn std::any::Any>>,
}

impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("pid", &self.pid)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("waiting_on", &self.waiting_on)
            .finish()
    }
}

impl Process {
    /// 建立新的行程（Ready 狀態）
    pub fn new(pid: Pid, name: &str, system_prompt: &str) -> Self {
        Self {
            pid,
            name: name.to_string(),
            state: ProcessState::Ready,
            mailbox: Mailbox::new(),
            waiting_on: None,
            system_prompt: system_prompt.to_string(),
            data: None,
        }
    }

    /// 阻塞等待某個 PID 完成
    pub fn block_on(&mut self, target: Pid) {
        self.state = ProcessState::Waiting;
        self.waiting_on = Some(target);
    }

    /// 喚醒（變回 Ready）
    pub fn wake(&mut self) {
        self.state = ProcessState::Ready;
        self.waiting_on = None;
    }

    /// 標記為執行中
    pub fn set_running(&mut self) {
        self.state = ProcessState::Running;
    }

    /// 標記為完成
    pub fn set_done(&mut self) {
        self.state = ProcessState::Done;
    }

    /// 是否在等待某個行程
    pub fn is_waiting(&self) -> bool {
        self.state == ProcessState::Waiting
    }

    /// 綁定任意資料（Node、Skill 等）
    pub fn set_data(&mut self, data: Box<dyn std::any::Any>) {
        self.data = Some(data);
    }

    /// 取出資料（downcast）
    pub fn take_data<T: 'static>(&mut self) -> Option<T> {
        self.data.take().and_then(|b| b.downcast::<T>().ok().map(|b| *b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid() {
        let p = Pid::new(42);
        assert_eq!(p.value(), 42);
        assert_eq!(format!("{}", p), "PID(42)");
    }

    #[test]
    fn test_process_state_transitions() {
        let mut p = Process::new(Pid::new(1), "test", "prompt");
        assert_eq!(p.state, ProcessState::Ready);

        p.set_running();
        assert_eq!(p.state, ProcessState::Running);

        p.block_on(Pid::new(0));
        assert_eq!(p.state, ProcessState::Waiting);
        assert_eq!(p.waiting_on, Some(Pid::new(0)));

        p.wake();
        assert_eq!(p.state, ProcessState::Ready);

        p.set_done();
        assert_eq!(p.state, ProcessState::Done);
    }

    #[test]
    fn test_process_mailbox() {
        let mut p = Process::new(Pid::new(1), "test", "prompt");
        p.mailbox.send("hello".to_string());
        p.mailbox.send("world".to_string());

        assert_eq!(p.mailbox.try_receive(), Some("hello".to_string()));
        assert_eq!(p.mailbox.try_receive(), Some("world".to_string()));
        assert_eq!(p.mailbox.try_receive(), None);
    }
}