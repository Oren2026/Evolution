//! ProcessTable — 行程表
//!
//! 管理系統中所有行程的生命週期。

use crate::kernel::Pid;
use crate::kernel::process::Process;
use crate::kernel::process::ProcessState;

/// 行程表
#[derive(Debug)]
pub struct ProcessTable {
    processes: Vec<Option<Process>>,
    next_pid: usize,
}

impl ProcessTable {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            next_pid: 1, // PID 0 保留為無效
        }
    }

    /// 新增行程（由 Kernel syscall 呼叫）
    pub fn spawn(&mut self, name: &str, system_prompt: &str) -> Pid {
        let pid = Pid::new(self.next_pid);
        // 確保 vec 有足夠空間（index = PID 值）
        while self.processes.len() <= self.next_pid {
            self.processes.push(None);
        }
        self.processes[self.next_pid] = Some(Process::new(pid, name, system_prompt));
        self.next_pid += 1;
        pid
    }

    /// 依 PID 取得可變參照
    pub fn get_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        let idx = pid.value();
        if idx == 0 || idx >= self.processes.len() {
            return None;
        }
        self.processes[idx].as_mut()
    }

    /// 依 PID 取得唯讀參照
    pub fn get(&self, pid: Pid) -> Option<&Process> {
        let idx = pid.value();
        if idx == 0 || idx >= self.processes.len() {
            return None;
        }
        self.processes[idx].as_ref()
    }

    /// 取得行程狀態
    pub fn state(&self, pid: Pid) -> Option<ProcessState> {
        self.get(pid).map(|p| p.state.clone())
    }

    /// 移除行程（Kernel 呼叫）
    pub fn remove(&mut self, pid: Pid) -> Option<Process> {
        let idx = pid.value();
        if idx == 0 || idx >= self.processes.len() {
            return None;
        }
        self.processes[idx].take()
    }

    /// 行程是否存在
    pub fn exists(&self, pid: Pid) -> bool {
        let idx = pid.value();
        idx > 0 && idx < self.processes.len() && self.processes[idx].is_some()
    }

    /// 行程數量
    pub fn len(&self) -> usize {
        self.processes.iter().filter(|p| p.is_some()).count()
    }

    /// 取得所有 Waiting 狀態的 PID（用於 Scheduler）
    pub fn waiting_pids(&self) -> Vec<Pid> {
        self.processes
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if i > 0 && p.as_ref().map_or(false, |proc| proc.state == ProcessState::Waiting) {
                    Some(Pid::new(i))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_get() {
        let mut table = ProcessTable::new();
        let pid1 = table.spawn("test", "sys prompt");
        assert_eq!(pid1.value(), 1);
        assert!(table.exists(pid1));
        assert_eq!(table.get(pid1).unwrap().name, "test");
    }

    #[test]
    fn test_multiple_spawn() {
        let mut table = ProcessTable::new();
        let p1 = table.spawn("p1", "");
        let p2 = table.spawn("p2", "");
        let p3 = table.spawn("p3", "");
        assert_eq!(p1.value(), 1);
        assert_eq!(p2.value(), 2);
        assert_eq!(p3.value(), 3);
        assert!(table.exists(p1));
        assert!(table.exists(p2));
        assert!(table.exists(p3));
    }

    #[test]
    fn test_state() {
        let mut table = ProcessTable::new();
        let pid = table.spawn("test", "");
        assert_eq!(table.state(pid), Some(ProcessState::Ready));
    }
}