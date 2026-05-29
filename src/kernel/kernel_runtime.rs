//! KernelRuntime — Planner → Kernel → Executor 統一進場點
//!
//! ## 設計
//! 這是 OS System 的高層 API，隱藏 Kernel/Scheduler 的細節，
//! 提供直接的三行程協調：spawn → send → wait → result
//!
//! ## 使用方式
//! ```rust,no_run
//! let mut kr = KernelRuntime::new();
//! kr.boot();
//!
//! // Planner 規劃
//! let manifest = kr.run_planner_sync("幫我建一個庫存管理系統");
//!
//! // Executor 執行
//! let results = kr.run_executor_sync(&manifest);
//!
//! println!("{:#?}", results);
//! ```

use crate::kernel::{
    Kernel, Pid, SysCall, SysCallKind, SysCallResult, ResultValue,
    NodeProcess, PlannerProcess, ExecutorProcess,
};
use crate::kernel::system_process::SystemProcess;
use crate::planner::manifest::Manifest;
use std::collections::HashMap;

/// Planner 行程資訊
#[derive(Debug)]
pub struct PlannerInfo {
    pub pid: Pid,
    pub system_prompt: String,
}

/// Executor 行程資訊
#[derive(Debug)]
pub struct ExecutorInfo {
    pub pid: Pid,
}

/// KernelRuntime — 高層 API，統一管理 Planner/Executor 行程

#[derive(Debug)]
pub struct KernelRuntime {
    kernel: Kernel,
    planner_pid: Option<Pid>,
    executor_pid: Option<Pid>,
}

impl KernelRuntime {
    /// 建立（尚未 boot）
    pub fn new() -> Self {
        Self {
            kernel: Kernel::new(),
            planner_pid: None,
            executor_pid: None,
        }
    }

    /// 開機
    pub fn boot(&mut self) {
        self.kernel.boot();
    }

    /// 是否已開機
    pub fn is_booted(&self) -> bool {
        self.kernel.is_booted()
    }

    // ─── Planner 操作 ───────────────────────────────────────────

    /// Spawn Planner 行程
    pub fn spawn_planner(&mut self) -> Pid {
        let prompt = "你是 Evolution OS 的 Planner，負責分析任務複雜度並產生分工藍圖。";
        let syscall = SysCall::spawn("planner", prompt, Pid::default());
        let result = self.kernel.syscall(syscall);
        let pid = result.expect_pid();
        self.planner_pid = Some(pid);
        pid
    }

    /// 對 Planner 發送任務
    pub fn send_to_planner(&mut self, task: &str) -> SysCallResult {
        let pid = self.planner_pid.expect("planner not spawned");
        let syscall = SysCall::send(pid, task.to_string(), Pid::default());
        self.kernel.syscall(syscall)
    }

    /// 從 Planner 接收 Manifest
    pub fn receive_from_planner(&mut self) -> Option<Manifest> {
        let pid = self.planner_pid.expect("planner not spawned");
        let syscall = SysCall::receive(pid, Pid::default());
        let result = self.kernel.syscall(syscall);

        if result.ok {
            if let ResultValue::Message(msg) = &result.value {
                if !msg.is_empty() {
                    return serde_json::from_str(msg).ok();
                }
            }
        }
        None
    }

    /// 等待 Planner 完成
    pub fn wait_planner(&mut self) -> SysCallResult {
        let pid = self.planner_pid.expect("planner not spawned");
        let syscall = SysCall::wait(pid, Pid::default());
        self.kernel.syscall(syscall)
    }

    // ─── Executor 操作 ──────────────────────────────────────────

    /// Spawn Executor 行程
    pub fn spawn_executor(&mut self) -> Pid {
        let prompt = "你是 Evolution OS 的 Executor，負責根據 Planner 的藍圖執行節點圖。";
        let syscall = SysCall::spawn("executor", prompt, Pid::default());
        let result = self.kernel.syscall(syscall);
        let pid = result.expect_pid();
        self.executor_pid = Some(pid);
        pid
    }

    /// 對 Executor 發送 Manifest
    pub fn send_to_executor(&mut self, manifest: &Manifest) -> SysCallResult {
        let pid = self.executor_pid.expect("executor not spawned");
        let json = serde_json::to_string(manifest).unwrap_or_default();
        let syscall = SysCall::send(pid, json, Pid::default());
        self.kernel.syscall(syscall)
    }

    /// 從 Executor 接收結果
    pub fn receive_from_executor(&mut self) -> Option<String> {
        let pid = self.executor_pid.expect("executor not spawned");
        let syscall = SysCall::receive(pid, Pid::default());
        let result = self.kernel.syscall(syscall);

        if result.ok {
            if let ResultValue::Message(msg) = &result.value {
                if !msg.is_empty() {
                    return Some(msg.clone());
                }
            }
        }
        None
    }

    /// 等待 Executor 完成
    pub fn wait_executor(&mut self) -> SysCallResult {
        let pid = self.executor_pid.expect("executor not spawned");
        let syscall = SysCall::wait(pid, Pid::default());
        self.kernel.syscall(syscall)
    }

    // ─── 組合工作流程 ───────────────────────────────────────────

    /// 完整流程：Planner → Executor → 結果
    ///
    /// 這是主要的展示用 API：
    /// 1. boot 若尚未
    /// 2. spawn planner + executor
    /// 3. 發送任務給 planner，接收 Manifest
    /// 4. 發送 Manifest 給 executor，接收執行結果
    /// 5. 兩個行程 exit
    pub fn run_full_pipeline(&mut self, task: &str) -> ExecutionResult {
        if !self.is_booted() {
            self.boot();
        }

        // Step 1: Spawn Planner
        let planner_pid = self.spawn_planner();

        // Step 2: Spawn Executor
        let executor_pid = self.spawn_executor();

        // Step 3: Planner 分析任務 → Manifest
        self.send_to_planner(task);
        let manifest = self.receive_from_planner()
            .expect("planner should return a manifest");
        self.wait_planner();

        // Step 4: 發送 Manifest 給 Executor
        self.send_to_executor(&manifest);
        let executor_output = self.receive_from_executor()
            .unwrap_or_else(|| "no output".to_string());
        self.wait_executor();

        // Step 5: Exit
        let exit_planner = SysCall::exit(planner_pid, 0);
        let exit_executor = SysCall::exit(executor_pid, 0);
        self.kernel.syscall(exit_planner);
        self.kernel.syscall(exit_executor);

        ExecutionResult {
            planner_pid,
            executor_pid,
            manifest,
            executor_output,
        }
    }

    // ─── 直接執行（sync 版本）──────────────────────────────────

    /// 直接執行 Planner（sync 版本，不走排程器）
    pub fn run_planner_sync(&mut self, task: &str) -> Manifest {
        let _pid = self.planner_pid.expect("planner not spawned");
        // 直接同步執行：使用 Manifest::from_task 生成藍圖
        let manifest = Manifest::from_task(task);
        println!("planner manifest: {}", manifest.to_json().unwrap());
        manifest
    }

    /// 直接執行 Executor（sync 版本）
    pub fn run_executor_sync(&mut self, manifest: &Manifest) -> String {
        let _pid = self.executor_pid.expect("executor not spawned");
        // 執行 GraphExecutor（sync）
        let exec_graph = crate::compiler::ExecutionGraph::from_manifest(manifest)
            .map_err(|e| format!("failed to build execution graph: {:?}", e));
        match exec_graph {
            Ok(eg) => {
                let mut graph = crate::node::MemoryGraph::new();
                let mut executor = crate::runtime::GraphExecutor::new();
                let results = executor.execute(&mut graph, &eg);
                let count = results.len();
                format!("executed {} nodes", count)
            }
            Err(e) => format!("execution graph error: {}", e),
        }
    }
}

impl Default for KernelRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// 執行結果
#[derive(Debug)]
pub struct ExecutionResult {
    pub planner_pid: Pid,
    pub executor_pid: Pid,
    pub manifest: Manifest,
    pub executor_output: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_runtime_spawn() {
        let mut kr = KernelRuntime::new();
        kr.boot();

        let planner_pid = kr.spawn_planner();
        let executor_pid = kr.spawn_executor();

        assert_ne!(planner_pid.value(), 0);
        assert_ne!(executor_pid.value(), 0);
        assert_ne!(planner_pid, executor_pid);
    }

    #[test]
    fn test_planner_sync() {
        let mut kr = KernelRuntime::new();
        kr.boot();

        let _ppid = kr.spawn_planner();
        let _epid = kr.spawn_executor();

        // 直接 sync 執行 Planner
        let manifest = kr.run_planner_sync("幫我建一個計數器網頁");
        assert!(!manifest.task.is_empty());
        println!("manifest: {}", manifest.to_json().unwrap());
    }

    #[test]
    fn test_executor_sync() {
        let mut kr = KernelRuntime::new();
        kr.boot();

        let _ppid = kr.spawn_planner();
        let _epid = kr.spawn_executor();

        let manifest = Manifest::from_task("幫我建一個計數器網頁");
        let output = kr.run_executor_sync(&manifest);
        println!("executor output: {}", output);
    }
}