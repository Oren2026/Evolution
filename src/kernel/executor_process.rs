//! ExecutorProcess — 將 GraphExecutor 包裝為 OS Process
//!
//! ## 職責
//! 1. 接收 Planner 的 Manifest（透過 mailbox）
//! 2. 根據 Manifest 生成 ExecutionGraph
//! 3. 執行 GraphExecutor，按 tier 順序執行節點
//! 4. 將結果寫入 output，供其他行程取用

use crate::kernel::{Kernel, Pid, ProcessState, SysCallKind, SysCallResult};
use crate::kernel::system_process::SystemProcess;
use crate::runtime::GraphExecutor;
use crate::node::MemoryGraph;
use crate::planner::manifest::Manifest;
use crate::compiler::ExecutionGraph;

/// 執行器行程 — 包裝 GraphExecutor
pub struct ExecutorProcess {
    pub pid: Pid,
    pub name: String,
    pub system_prompt: String,
    executor: GraphExecutor,
    graph: MemoryGraph,
    exec_graph: Option<ExecutionGraph>,
    results: Vec<String>,
}

impl ExecutorProcess {
    pub fn new(pid: Pid) -> Self {
        Self {
            pid,
            name: "executor".to_string(),
            system_prompt: "你是一個執行器，負責根據 Planner 的藍圖執行節點圖".to_string(),
            executor: GraphExecutor::new(),
            graph: MemoryGraph::new(),
            exec_graph: None,
            results: vec![],
        }
    }

    /// 接收 PlannerManifest，初始化執行圖
    pub fn load_manifest(&mut self, manifest: &Manifest) -> Result<(), String> {
        let exec_graph = ExecutionGraph::from_manifest(manifest)
            .map_err(|e| format!("failed to build execution graph: {:?}", e))?;

        self.exec_graph = Some(exec_graph);
        Ok(())
    }

    /// 執行一次（每 tick 一次）
    pub fn execute_tick(&mut self) {
        if let Some(ref mut eg) = self.exec_graph {
            let results = self.executor.execute(&mut self.graph, eg);
            for r in results {
                self.results.push(r.output);
            }
        }
    }

    /// 是否已執行完畢
    pub fn is_complete(&self) -> bool {
        self.exec_graph.is_some()
    }

    /// 取得結果
    pub fn get_results(&self) -> &[String] {
        &self.results
    }
}

impl SystemProcess for ExecutorProcess {
    fn name(&self) -> &str {
        &self.name
    }

    fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    fn handle(&mut self, message: &str, _kernel: &mut Kernel) -> Result<String, String> {
        // 嘗試解析 Manifest JSON
        let manifest: Manifest = serde_json::from_str(message)
            .map_err(|e| format!("invalid manifest: {}", e))?;

        self.load_manifest(&manifest)?;

        // 執行
        self.execute_tick();

        let output = format!("executed {} nodes, {} results",
            self.graph.list_node_ids().len(),
            self.results.len()
        );
        Ok(output)
    }

    fn is_done(&self) -> bool {
        self.is_complete()
    }
}