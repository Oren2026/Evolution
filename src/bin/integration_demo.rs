//! integration_demo.rs — Planner→Kernel→Executor 端對端展示
//!
//! ## 執行方式
//! ```
//! cargo run --bin integration_demo
//! ```

use evolution_os::kernel::{Kernel, Pid, SysCall};
use evolution_os::kernel::kernel_runtime::KernelRuntime;
use evolution_os::planner::manifest::Manifest;
use evolution_os::runtime::GraphExecutor;
use evolution_os::node::MemoryGraph;
use evolution_os::compiler::ExecutionGraph;

fn print_separator(title: &str) {
    println!();
    println!("══════════════════════════════════════════");
    println!("  {}", title);
    println!("══════════════════════════════════════════");
}

fn demo_kernel_runtime() {
    print_separator("KernelRuntime API 展示");

    let mut kr = KernelRuntime::new();
    kr.boot();

    println!("[boot] Kernel 已開機");

    // Spawn Planner
    let planner_pid = kr.spawn_planner();
    println!("[spawn] Planner PID: {:?}", planner_pid);

    // Spawn Executor
    let executor_pid = kr.spawn_executor();
    println!("[spawn] Executor PID: {:?}", executor_pid);

    // 執行 Planner（sync）
    let task = "幫我建一個計數器網頁，點擊按鈕數字 +1";
    let manifest = kr.run_planner_sync(task);
    println!("[planner] manifest.task = {}", manifest.task);
    println!("[planner] stage = {:?}", manifest.stage);
    println!("[planner] estimated_nodes: {}", manifest.estimated_nodes.len());
    for (i, node) in manifest.estimated_nodes.iter().enumerate() {
        println!("  {}. {} (role: {}, depends: {:?})", i + 1, node.id, node.role, node.depends_on);
    }

    println!();

    // 執行 Executor（sync）
    let output = kr.run_executor_sync(&manifest);
    println!("[executor] output: {}", output);

    println!();
    println!("[OK] KernelRuntime 端對端流程完成");
}

fn demo_kernel_direct() {
    print_separator("Kernel Direct SysCall 展示");

    let mut kernel = Kernel::new();
    kernel.boot();

    println!("[boot] Kernel 已開機");
    println!();

    // Spawn 行程
    let syscall1 = SysCall::spawn("planner", "Planner 行程", Pid::default());
    let r1 = kernel.syscall(syscall1);
    let planner_pid = r1.expect_pid();
    println!("[spawn] planner PID = {:?}", planner_pid.value());

    let syscall2 = SysCall::spawn("executor", "Executor 行程", Pid::default());
    let r2 = kernel.syscall(syscall2);
    let executor_pid = r2.expect_pid();
    println!("[spawn] executor PID = {:?}", executor_pid.value());

    println!();

    // Send Message
    let send_syscall = SysCall::send(planner_pid, "你好 Planner！".to_string(), Pid::default());
    let result = kernel.syscall(send_syscall);
    println!("[send] to planner -> ok={}", result.ok);

    // Receive Message
    let recv_syscall = SysCall::receive(planner_pid, Pid::default());
    let result = kernel.syscall(recv_syscall);
    println!("[receive] from planner: ok={}", result.ok);

    println!();

    // Wait
    let wait_syscall = SysCall::wait(planner_pid, Pid::default());
    let result = kernel.syscall(wait_syscall);
    println!("[wait] planner done: ok={}", result.ok);

    // Exit
    let exit_syscall = SysCall::exit(planner_pid, 0);
    let result = kernel.syscall(exit_syscall);
    println!("[exit] planner: ok={}", result.ok);

    println!();
    println!("[OK] Kernel Direct SysCall 展示完成");
}

fn demo_execution_graph() {
    print_separator("ExecutionGraph 建構展示");

    let task = "幫我建一個天氣查詢網頁";
    let manifest = Manifest::from_task(task);

    println!("[manifest] task: {}", manifest.task);
    println!("[manifest] stage = {:?}", manifest.stage);
    println!("[manifest] estimated_nodes: {}", manifest.estimated_nodes.len());
    for (i, n) in manifest.estimated_nodes.iter().enumerate() {
        println!("  {}: {} (role: {:?})", i + 1, n.id, n.role);
    }

    println!();

    match ExecutionGraph::from_manifest(&manifest) {
        Ok(graph) => {
            println!("[graph] built successfully");
            let order = graph.execution_order();
            println!("[graph] {} tiers:", order.len());
            for (i, tier) in order.iter().enumerate() {
                println!("  tier {}: {:?}", i, tier);
            }

            // 執行
            let mut mem_graph = MemoryGraph::new();
            let executor = GraphExecutor::new();
            let results = executor.execute(&mut mem_graph, &graph);
            println!("[execute] {} nodes completed", results.len());
        }
        Err(e) => {
            println!("[graph] build failed: {:?}", e);
        }
    }

    println!();
    println!("[OK] ExecutionGraph 展示完成");
}

fn main() {
    println!();
    println!("╔══════════════════════════════════════════╗");
    println!("║   Evolution OS — Integration Demo        ║");
    println!("║   Planner → Kernel → Executor           ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // Demo 1: KernelRuntime（高層 API）
    demo_kernel_runtime();

    println!();

    // Demo 2: Kernel Direct SysCall
    demo_kernel_direct();

    println!();

    // Demo 3: ExecutionGraph
    demo_execution_graph();

    println!();
    println!("══════════════════════════════════════════");
    println!("  All demos completed successfully!");
    println!("══════════════════════════════════════════");
    println!();
}