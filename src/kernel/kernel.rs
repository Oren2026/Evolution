//! Kernel — Evolution OS 核心
//!
//! 唯一進場點：Kernel::syscall()
//!
//! 所有行程間的溝通都必須經過這裡。

pub mod mailbox;
pub mod process;
pub mod process_table;
pub mod scheduler;
pub mod syscall;

pub use mailbox::Mailbox;
pub use process::{Pid, Process, ProcessState};
pub use process_table::ProcessTable;
pub use scheduler::Scheduler;
pub use syscall::{SysCall, SysCallKind, SysCallResult, ResultValue};

use crate::kernel::Pid;
use crate::kernel::process::ProcessState;
use crate::kernel::syscall::SysCallKind;

/// 內核 — 系統呼叫的單一進入點
///
/// ## 設計原則
/// 1. **唯一進場**：行程不直接呼叫彼此，全部經由 `Kernel::syscall()`
/// 2. **無共享狀態**：行程之間透過 Mailbox 傳訊息，不共享記憶體
/// 3. **Blocking Receive**：Receive 是唯一會 block 行程的 syscall
///
/// ## SysCall 類型
/// - `Spawn` — 建立新行程（Kernel 自動呼叫，不會 block）
/// - `Send`  — 傳訊息給某 PID（永遠不 block）
/// - `Receive` — 接收訊息（若信箱空則 block）
/// - `Wait`   — 等待某 PID 完成（block 直到目標 Done）
/// - `Exit`   — 行程結束
#[derive(Debug)]
pub struct Kernel {
    /// 行程表
    table: ProcessTable,
    /// 排程器
    scheduler: Scheduler,
    /// 目前正在執行的 PID（None = idle）
    running: Option<Pid>,
    /// 系統是否已啟動
    booted: bool,
}

impl Kernel {
    /// 建立新內核
    pub fn new() -> Self {
        Self {
            table: ProcessTable::new(),
            scheduler: Scheduler::new(),
            running: None,
            booted: false,
        }
    }

    /// 開機
    pub fn boot(&mut self) {
        self.booted = true;
        self.running = None;
    }

    /// 系統是否已啟動
    pub fn is_booted(&self) -> bool {
        self.booted
    }

    /// 取得目前執行中的 PID
    pub fn running_pid(&self) -> Option<Pid> {
        self.running
    }

    // ─── SysCall 實作 ───────────────────────────────────────────────

    /// 系統呼叫 — 行程與 OS 溝通的唯一管道
    pub fn syscall(&mut self, call: SysCall) -> SysCallResult {
        if !self.booted {
            return SysCallResult::err("kernel not booted");
        }

        match call.kind {
            SysCallKind::Spawn { name, system_prompt } => {
                let pid = self.do_spawn(&name, &system_prompt);
                SysCallResult::ok(ResultValue::Pid(pid))
            }

            SysCallKind::Send { target, content } => {
                self.do_send(call.caller, target, content)
            }

            SysCallKind::Receive => {
                self.do_receive(call.caller)
            }

            SysCallKind::Wait { target } => {
                self.do_wait(call.caller, target)
            }

            SysCallKind::Exit { code } => {
                self.do_exit(call.caller, code)
            }
        }
    }

    /// Spawn：建立新行程
    fn do_spawn(&mut self, name: &str, system_prompt: &str) -> Pid {
        let pid = self.table.spawn(name, system_prompt);
        self.scheduler.enqueue(pid);
        pid
    }

    /// Send：傳訊息
    fn do_send(&mut self, caller: Pid, target: Pid, content: String) -> SysCallResult {
        // 驗證目標存在
        if !self.table.exists(target) {
            return SysCallResult::err(&format!("send: target {} does not exist", target.value()));
        }

        // 投遞到目標信箱
        if let Some(p) = self.table.get_mut(target) {
            p.mailbox.send(content);
            // 若目標正在等訊息，喚醒它
            if p.state == ProcessState::Waiting {
                p.wake();
                self.scheduler.enqueue(target);
            }
        }

        SysCallResult::ok(ResultValue::None)
    }

    /// Receive：接收訊息（blocking）
    fn do_receive(&mut self, caller: Pid) -> SysCallResult {
        let process = match self.table.get_mut(caller) {
            Some(p) => p,
            None => return SysCallResult::err("receive: caller process not found"),
        };

        // 檢查信箱
        match process.mailbox.try_receive() {
            Some(msg) => SysCallResult::ok(ResultValue::Message(msg)),
            None => {
                // 信箱為空，block
                process.block_on(Pid::default()); // 等待任意來源
                SysCallResult::ok(ResultValue::Message(String::new()))
            }
        }
    }

    /// Wait：等待目標完成
    fn do_wait(&mut self, caller: Pid, target: Pid) -> SysCallResult {
        let process = match self.table.get_mut(caller) {
            Some(p) => p,
            None => return SysCallResult::err("wait: caller process not found"),
        };

        // 檢查目標狀態
        if let Some(state) = self.table.state(target) {
            if state == ProcessState::Done {
                return SysCallResult::ok(ResultValue::Done(target));
            }
        } else {
            return SysCallResult::err(&format!("wait: target {} not found", target.value()));
        }

        // 尚未完成，block
        process.block_on(target);
        SysCallResult::ok(ResultValue::Done(target))
    }

    /// Exit：行程結束
    fn do_exit(&mut self, caller: Pid, code: i32) -> SysCallResult {
        if let Some(p) = self.table.get_mut(caller) {
            p.set_done();
            self.running = None;
        }
        SysCallResult::ok(ResultValue::ExitCode(code))
    }

    // ─── 排程 ──────────────────────────────────────────────────────

    /// 取得下一個應執行的 PID
    pub fn schedule(&mut self) -> Option<Pid> {
        // 標記目前行程為 Ready（若仍在執行）
        if let Some(running_pid) = self.running {
            if let Some(p) = self.table.get_mut(running_pid) {
                if p.state == ProcessState::Running {
                    p.state = ProcessState::Ready;
                }
            }
        }

        // 更新排程器
        self.scheduler.update(&self.table);

        // 取下一個
        let next = self.scheduler.next(&self.table);
        if let Some(pid) = next {
            if let Some(p) = self.table.get_mut(pid) {
                p.set_running();
            }
            self.running = Some(pid);
        } else {
            self.running = None;
        }

        next
    }

    /// 執行一個時間切片（一次 schedule + 一次 syscall）
    /// 回傳是否還有行程在跑
    pub fn tick(&mut self) -> bool {
        self.schedule();
        self.running.is_some()
    }

    // ─── 查詢 ──────────────────────────────────────────────────────

    /// 行程表大小
    pub fn process_count(&self) -> usize {
        self.table.len()
    }

    /// 取得行程狀態
    pub fn state(&self, pid: Pid) -> Option<ProcessState> {
        self.table.state(pid)
    }

    /// 列出所有行程狀態
    pub fn ps(&self) -> Vec<(Pid, String, ProcessState)> {
        (1..self.table.len() + 1)
            .filter_map(|i| {
                let pid = Pid::new(i);
                self.table.get(pid).map(|p| (p.pid, p.name.clone(), p.state.clone()))
            })
            .collect()
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spawn(call: SysCallKind) -> SysCall {
        SysCall { caller: Pid::default(), kind: call }
    }

    #[test]
    fn test_kernel_boot_spawn() {
        let mut k = Kernel::new();
        k.boot();

        let r = k.syscall(make_spawn(SysCallKind::Spawn {
            name: "planner".into(),
            system_prompt: "You are a planner".into(),
        }));
        assert!(r.ok);
        let pid = match r.value {
            ResultValue::Pid(p) => p,
            _ => panic!("expected Pid"),
        };
        assert_eq!(k.state(pid), Some(ProcessState::Ready));
        assert_eq!(k.process_count(), 1);
    }

    #[test]
    fn test_kernel_send_and_receive() {
        let mut k = Kernel::new();
        k.boot();

        let planner = k.syscall(make_spawn(SysCallKind::Spawn {
            name: "planner".into(),
            system_prompt: "planner".into(),
        })).value;
        let planner_pid = match planner { ResultValue::Pid(p) => p, _ => panic!() };

        let worker = k.syscall(make_spawn(SysCallKind::Spawn {
            name: "worker".into(),
            system_prompt: "worker".into(),
        })).value;
        let worker_pid = match worker { ResultValue::Pid(p) => p, _ => panic!() };

        // Worker 先嘗試 receive（會 block，但這裡測試不真的 block）
        let r = k.syscall(SysCall { caller: worker_pid, kind: SysCallKind::Receive });
        // 信箱空，這個實作會回 empty message（真實 OS 會真的 block）
        assert!(r.ok);

        // Planner send to worker
        let r = k.syscall(SysCall {
            caller: planner_pid,
            kind: SysCallKind::Send {
                target: worker_pid,
                content: "task: analyze this".into(),
            },
        });
        assert!(r.ok);

        // Worker receive now
        let r = k.syscall(SysCall { caller: worker_pid, kind: SysCallKind::Receive });
        assert!(r.ok);
        match r.value {
            ResultValue::Message(msg) => assert_eq!(msg, "task: analyze this"),
            _ => panic!("expected Message"),
        }
    }

    #[test]
    fn test_kernel_wait_and_exit() {
        let mut k = Kernel::new();
        k.boot();

        let child = k.syscall(make_spawn(SysCallKind::Spawn {
            name: "child".into(),
            system_prompt: "child".into(),
        })).value;
        let child_pid = match child { ResultValue::Pid(p) => p, _ => panic!() };

        let parent = k.syscall(make_spawn(SysCallKind::Spawn {
            name: "parent".into(),
            system_prompt: "parent".into(),
        })).value;
        let parent_pid = match parent { ResultValue::Pid(p) => p, _ => panic!() };

        // Parent wait for child
        let r = k.syscall(SysCall {
            caller: parent_pid,
            kind: SysCallKind::Wait { target: child_pid },
        });
        assert!(r.ok); // 不會真的 block，因為馬上 child 還沒 Done

        // Child exit
        let r = k.syscall(SysCall {
            caller: child_pid,
            kind: SysCallKind::Exit { code: 0 },
        });
        assert!(r.ok);

        // Now child should be Done
        assert_eq!(k.state(child_pid), Some(ProcessState::Done));
    }

    #[test]
    fn test_kernel_schedule() {
        let mut k = Kernel::new();
        k.boot();

        let p1 = match k.syscall(make_spawn(SysCallKind::Spawn {
            name: "p1".into(),
            system_prompt: "".into(),
        })).value { ResultValue::Pid(p) => p, _ => panic!() };

        let p2 = match k.syscall(make_spawn(SysCallKind::Spawn {
            name: "p2".into(),
            system_prompt: "".into(),
        })).value { ResultValue::Pid(p) => p, _ => panic!() };

        // First tick: should pick p1
        let next = k.schedule();
        assert_eq!(next, Some(p1));
        assert_eq!(k.running_pid(), Some(p1));

        // Second tick: should pick p2
        let next = k.schedule();
        assert_eq!(next, Some(p2));
        assert_eq!(k.running_pid(), Some(p2));
    }

    #[test]
    fn test_kernel_not_booted() {
        let mut k = Kernel::new();
        let r = k.syscall(make_spawn(SysCallKind::Spawn {
            name: "test".into(),
            system_prompt: "".into(),
        }));
        assert!(!r.ok);
        assert!(r.error.is_some());
    }
}