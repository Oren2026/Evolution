//! Scheduler — 排程器
//!
//! 最小 FIFO 排程：
//! 1. 從 Ready 佇列依序取出
//! 2. 若該行程的依賴已 Done，喚醒並標記為 Ready
//! 3. 下一個 Running = 佇列最前端

use crate::kernel::Pid;
use crate::kernel::ProcessState;
use crate::kernel::process_table::ProcessTable;

/// 排程器 — 簡單 FIFO
#[derive(Debug)]
pub struct Scheduler {
    ready_queue: Vec<Pid>,
    valid_pids: Vec<Pid>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: Vec::new(),
            valid_pids: Vec::new(),
        }
    }

    /// 更新排程：
    /// - 將已完成的 Waiting 行程喚醒為 Ready
    /// - 清理已不在表格中的 PID
    pub fn update(&mut self, table: &mut ProcessTable) {
        // 喚醒那些等待目標已完成的行程
        for pid in table.waiting_pids() {
            if let Some(p) = table.get(pid) {
                if let Some(target) = p.waiting_on {
                    if let Some(state) = table.state(target) {
                        if state == ProcessState::Done {
                            // 目標已完成，喚醒這個行程
                            if let Some(target_proc) = table.get_mut(pid) {
                                target_proc.wake();
                                if !self.ready_queue.contains(&pid) {
                                    self.ready_queue.push(pid);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 清理不再存在的 PID
        self.ready_queue.retain(|&pid| table.exists(pid));
    }

    /// 取出下一個應執行的 PID（已為 Ready 的最前端）
    /// 注意：不會呼叫 update()，由 Kernel.schedule() 統一管理 update
    pub fn next(&mut self, table: &mut ProcessTable) -> Option<Pid> {
        // 找第一個仍為 Ready 的 PID（不改變佇列結構）
        for i in 0..self.ready_queue.len() {
            let pid = self.ready_queue[i];
            if table.exists(pid) && table.state(pid) == Some(ProcessState::Ready) {
                self.ready_queue.remove(i);
                return Some(pid);
            }
        }
        None
    }

    /// 手動加入 Ready 佇列（由 Kernel 呼叫）
    pub fn enqueue(&mut self, pid: Pid) {
        if !self.ready_queue.contains(&pid) {
            self.ready_queue.push(pid);
        }
        if !self.valid_pids.contains(&pid) {
            self.valid_pids.push(pid);
        }
    }

    /// 同步 valid_pids（由 Kernel.schedule() 統一呼叫）
    pub fn sync_valid_pids(&mut self, table: &ProcessTable) {
        self.valid_pids.retain(|&pid| table.exists(pid));
    }

    /// 排程佇列長度
    pub fn len(&self) -> usize {
        self.ready_queue.len()
    }

    /// 是否為空
    pub fn is_empty(&self) -> bool {
        self.ready_queue.is_empty()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}