# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.3.1] — 2026-05-29

### Added

- **`index.html`** — 系統規格書 landing page（Evolution OS 介紹、Layer 架構、CLI Commands、安裝說明）
- **整合測試**（127 tests passed, 0 failures）：
  - `tests/graph_executor.rs` — 7 tests，MemoryGraph 核心功能覆蓋
  - `tests/chain_discovery.rs` — 7 tests，呼叫鏈探索邊界條件
  - `tests/runtime_executor.rs` — 7 tests，執行流程與依賴排序
- **`.gitignore`** — 排除 `target/`，不再推送編譯產物進 repo
- **Release Workflow v0.2.3** — 完整 GitHub Actions 自動 release：
  - 4 平台編譯（Linux x64 / macOS x64+arm64 / Windows x64）
  - 直接上傳 binaries 到 GitHub Release Assets（`gh release upload`）
  - Windows 打包使用 PowerShell（`pwsh`），避免 bash 語法相容性問題

### Fixed

- 所有 compiler warnings（unused imports, unused variables, dead code）
- `kernel_runtime.rs`: 移除broken doctest（無法在私有型別context中使用）
- `node/registry.rs`: 補回 `use std::any::Any`（`as_any()` trait impl 需要）
- git push時 `target/` 進repo導致 388MB commit → 建立 `.gitignore` + 以後再不進repo
- Release workflow：移除 `actions/upload-artifact` / `actions/download-artifact`（CI artifacts 1天後消失）→ 改用 `gh release upload` 直接當 Release assets

### Changed

- `Cargo.toml` `--features llm` 移除：release build 不編譯 LLM相依（省記憶體、避免 compiler panic）
- Release workflow：簡化為 build → package → upload 三步驟，不再有 race condition 問題

---

## [0.3.0] — 2026-05-28

### Added

- **`kernel` module** — OS System 核心（sync Rust，tokio-less）
  - `mod.rs`: Kernel（本體）— `syscall()` 單一進場點，422 行含完整測試
  - `process.rs`: Process / ProcessState / Pid（含 Debug impl，147 行）
  - `mailbox.rs`: Mailbox（FIFO 訊息佇列，57 行）
  - `process_table.rs`: ProcessTable（index-based，PID=index，122 行）
  - `scheduler.rs`: Scheduler（FIFO，不遞迴 update，93 行）
  - `syscall.rs`: SysCallKind（Spawn/Send/Receive/Wait/Exit，93 行）
  - `system_process.rs`: SystemProcess trait + NodeProcess + PlannerProcess（148 行）

- **kernel 測試**：11 tests passed，0 failures

### Architecture Change

- 從「直線 Pipe」重構為「OS 架構」：
  - 直線 Pipe（旧）：Planner → Compiler → Executor，直接函式呼叫
  - OS Kernel（新）：所有節點都是 Process，透過 `Kernel.syscall()` 互動

### Fixed

- `scheduler.rs`: `next()` 遞迴呼叫 `update()` 造成 retain twice — 加入 `sync_valid_pids()` 由 `Kernel.schedule()` 統一呼叫
- `process_table.rs`: index 0 保留（無效 PID），`spawn()` 時使用 while loop 而非 resize
- `kernel/mod.rs`: `do_receive()` 移除不必要的 `block_on()` 呼叫
- `decision.rs`: `ComplexityMetrics` / `DispatchDecision` / `OptimizerPrompt` 加 `#[derive(Default)]`

---

## [0.2.0] — 2026-05-27

### Added

- **期末文件**：SPEC.md 規格書 + REPORT.md 報告 + CHANGELOG.md + VERSION_CONTROL.md
- **系統架構圖**（ASCII）：Planner Stage Flow + Dispatch Decision 閾值判定樹 + 領域關鍵詞對照表
- **期末展示大綱**：Demo 項目（終端指令）+ 文件展示清單

---

## [0.1.0] — 2026-05-27

### Added

- **`planner` module** — 任務規劃與分工決策核心
  - `stages.rs`: Stage 枚舉（S1Confirming / S2Analyzing / S3Planning / Complete）
  - `decision.rs`: ComplexityMetrics、DispatchDecision、WorkMode（Fork/Solo）
  - `manifest.rs`: PlannerManifest、Requirement、QuestionItem、EstimatedNode

- **`planner_cli` binary** — 命令行入口
  - 直接模式：`cargo run --bin planner_cli -- "<任務>"` → 終端分析報告 + JSON
  - 互動模式（`--interactive`）：多行輸入

- **整合測試**：`tests/planner_decision.rs`（6 cases，驗證分工閾值）

- **`VERSION_CONTROL.md`** — 版本控制規劃文件

### Changed

- `src/lib.rs`: 加入 `pub mod planner;`
- `Cargo.toml`: 新增 `chrono = "0.4"` dependency

### Fixed

- `decision.rs`: `count_domain_diversity` 数组长度不一导致的 type inference 错误
- `decision.rs`: `tech_terms` 变量名拼写错误
- `planner_decision.rs`: `frontend` tag 识别失败（新增 "網站/web/頁面" keyword）

---

## [0.0.0] — 2026-05-?? (Project Initialized)

### Added

- Initial project structure: `node`, `chain`, `skill`, `runtime`, `model`, `storage`, `evo`
- OllamaBackend for model dispatching
- MemoryGraph for call chain tracking
- Executor for sequential node execution
- SkillRegistry and built-in skill nodes
- JsonStorage for graph persistence