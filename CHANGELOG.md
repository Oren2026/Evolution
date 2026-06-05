# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.6.0] — 2026-06-05

### Added

- **`EventGenerator` — LLM 驅動的系統事件解讀**：
  - `src/system/event_generator.rs`（126 lines）
  - `interpret_transition(task_name, from_stage, to_stage, backend)` — 階段轉換時使用 gemma4:e2b 生成事件描述
  - `interpret_transition_with_model()` — 指定模型版本
  - 回傳 `Option<String>`，失敗時 caller 可 fallback 到規則版本

- **`EventEntry.interpretation` 欄位**：
  - `src/system/state_board.rs` — EventEntry 新增 `interpretation: Option<String>` 欄位
  - `with_interpretation()` builder helper
  - `update_stage()` 在 Done/Blocked 時自動呼叫 EventGenerator 附加 LLM 解讀

- **`StateBoard::understand_events()` — 主動理解**：
  - 讀取所有未送達事件 + 任務狀態
  - 送入 LLM 生成繁體中文系統建議（100-200 字）
  - 無事件時快速回覆「沒有待處理事件。系統正常運行。」
  - 需要明確傳入 `backend: &dyn ModelDispatcher`（explicit API）

- **`Shell → StateBoard 追蹤`（`execute_evolution_cli()` 重構）：
  - `src/commands/shell.rs` — 執行前建立 TaskEntry（Executing）
  - 執行成功 → Done，失敗 → Blocked（帶錯誤 note）
  - StateBoard 自動持久化到 `~/.evolution/state_board.json`
  - `extract_task_name_from_cli()` 從 CLI 字串推斷任務名

### Changed

- **`StateBoard::update_stage()` 簽名**：第四參數 `backend: Option<&dyn ModelDispatcher>`
- **`src/system/state_board_storage.rs` 測試**：更新所有 `update_stage()` 呼叫為 4 參數
- **`src/lib.rs` / `src/system/mod.rs`**：export EventGenerator
- **`EventGenerator::default_model()`**：新增 public accessor（從 private 提升）

### Test Status

- `cargo build`：通過（~20 warnings，0 errors）
- `cargo test`：全部通過（lib + 2 bins + doc-tests）

---

## [0.5.0] — 2026-06-05

### Added

- **`StateBoard` — 系統狀態表**：
  - `src/system/state_board.rs` — TaskEntry、PresenceEntry、EventEntry、TaskStage、StateBoard 本體
  - `src/system/state_board_storage.rs` — 持久化至 `~/.evolution/state_board.json`
  - `BoardSummary`、`StateBoardSnapshot` 結構
  - 自動事件生成（Done → EventLevel::Done，Blocked → EventLevel::NeedsUser）

- **`evolution board` CLI 命令**：
  - `evolution board` — 顯示任務概覽（依階段分組）
  - `evolution board events` — 顯示未送達事件
  - `evolution board <uuid>` — 單一任務詳情（含階段歷史）
  - `evolution board clear` — 清除已完成任務
  - 實作：`src/commands/board.rs`（202 lines）

- **`IntentCommandDB` — 意圖命令資料庫**：
  - `src/commands/intent_router.rs`（514 lines）
  - `IntentCommandDB`：`CommandDef` + `CommandSlot` + `aliases` + `examples`
  - `IntentRouter`：關鍵字匹配（`route_keyword`）+ LLM 意圖分類（`route_with_model`）
  - 內建 13 個命令：board（4）、planner（2）、system（3）、dev（1）、query（2）
  - 預設使用 `gemma4:e2b` 模型做意圖分類

### Changed

- **`src/commands/shell.rs`**：重構為 IntentRouter 主動路由 + execute_evolution_cli() 直接執行 Evolution CLI，移除重複的 classify_intent/init 呼叫
- **`src/main.rs`**：`Board` subcommand 加入 CLI match arm
- **`src/commands/mod.rs`**：公開 `board` 模組
- **`src/lib.rs`**：export StateBoard 相關類型

### Fixed

- **`src/system/state_board_storage.rs`**：`super::state_board` → `crate::system::state_board`（修正超模組 import 路徑，修復編譯阻擋）
- `crate::model` → `evolution_os::model`（bin crate 正確路徑）
- `ModelRequest::new(model, prompt)` 參數順序（原本 3 個參數，實際只需要 2 個）
- `&[String]` pattern matching 改用 `(bool, Option<&str>)` 元組避免 slice パターン不相容
- `EventLevel::Error` 不存在，改用 `Warning` 替代

### Test Status

- `cargo build`：通過（28 warnings，0 errors）
- `cargo test`：108 tests passed, 0 failures

---

## [0.3.2] — 2026-06-03

### Added

- **`shell` mode — 真實命令執行**：
  - `handle_system`：真正執行 `open` 開檔（支援路徑擴展、應用程式名自動補 `.app`）
  - `handle_evolution`：真正呼叫 CLI 命令（`analyze` / `new` / `status` / `list-skills` / `init`），不再只是印 placeholder

### Changed

- **`classify_intent` prompt**：移除 `opencode` 分類（已拔除）
- **Intent 分類数**：5 → 4（system / evolution / calendar / general）
- **`handle_opencode`**：標記為 stub，輸出 `[任務委派中...]` 不再提供

### Removed

- **`opencode` intent 分支**：不適合 Evolution OS 的單一 CLI 模型，移除
- **`find_game()`**：功能已整合进 `exec_open()`，不再需要

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