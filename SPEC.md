# Evolution OS — 系統規格書
# Evolution Compiler — 軟體工程Framework

**版本**: v0.3.0 (DRAFT)
**日期**: 2026-05-28
**類型**: 學期專案成果文件

---

## 1. 系統背景與目的

### 1.1 問題動機

現有 AI 程式碼生成工具（如 Copilot、Codex）在面對**複雜、多領域的軟體系統**時，常有以下問題：

- 單一 AI prompt 無法同時處理：前端、後端、資料庫、認證等多面向任務
- 推理過程不透明：難以理解 AI 為何做出某個分工決策
- 缺乏顯性的需求收斂：還沒確認需求就先實作，方向錯誤後難以修正
- 無法表達「我的意圖是什麼」：只能寫規格，無法讓 AI 理解背後的意圖

### 1.2 解決方向：Evolution

**Evolution OS** 是一個以 **意圖驅動（Intent-Driven）** 為核心的 AI 推理/協作工具。

核心理念：
1. **Input**: 任務描述（自然語言）
2. **Analyze**: 分析問題的複雜度、領域多樣性、推理分支數
3. **Dispatch**: 根據分析結果，決定 Solo 或 Fork 模式
4. **Plan**: 輸出完整的分工 Manifest（問題確認、需求清單、節點結構）
5. **Execute**: 根據 Manifest 調度相應的 AI 節點執行

最終目標：**「能夠給 AI 更好輸入的軟體」**，而不是「寫代碼的軟體」。

---

## 2. 系統架構圖

```
┌─────────────────────────────────────────────────────────────────┐
│                      Evolution OS                              │
│                                                                │
│   Input: "幫我建一個庫存管理系統"                                 │
│                          ↓                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    PLANNER                              │   │
│  │   ┌──────────┐  ┌──────────┐  ┌───────────────────┐     │   │
│  │   │  S1      │→ │  S2      │→ │  S3                │     │   │
│  │   │  確認需求 │  │  分析問題 │  │  規劃派工           │     │   │
│  │   │(Converge)│  │(Analyze) │  │  (Dispatch Plan)   │     │   │
│  │   └──────────┘  └──────────┘  └───────────────────────┘     │   │
│  │         ↓               ↓               ↓                      │   │
│  │   questions[]    complexity    ┌────────────────────┐        │   │
│  │                         Metrics  │  PlannerManifest │        │   │
│  │                              └────────────────────┘        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                               ↓                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     EXECUTOR                              │  │
│  │  ┌────────┐   ┌────────┐   ┌────────┐                     │  │
│  │  │ Planner │   │ Front  │   │  QA    │  ...               │  │
│  │  │ Node   │──→│ Node   │──→│ Node   │                     │  │
│  │  └────────┘   └────────┘   └────────┘                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                               ↓                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  MEMORY GRAPH                            │  │
│  │   呼叫鏈追蹤：Leaf → ... → Root                          │  │
│  └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.1 Planner Stage Flow（核心流程）

```
輸入：task description
   │
   ├─▶ S1: 確認需求（questions[]）
   │       ├─ 檢測需求模糊點（技術栈、部署環境、權限範圍）
   │       └─ 輸出：待確認問題清單
   │
   ├─▶ S2: 分析問題
   │       ├─ ComplexityMetrics.estimate_from_task()
   │       │   ├─ reasoning_branches（推理分支數）
   │       │   ├─ domain_diversity（領域多樣性）
   │       │   └─ context_complexity（語境複雜度）
   │       └─ 輸出：複雜度指標
   │
   └─▶ S3: 規劃派工
           ├─ WorkMode.decide(metrics)
           │   ├─ Solo: branches ≤ 2 AND diversity ≤ 1 AND complexity ≤ 0.6
           │   └─ Fork: branches > 2 OR diversity > 1 OR complexity > 0.6
           ├─ DispatchDecision.from_task()
           │   ├─ estimated_nodes（預估節點數 1-6）
           │   ├─ domain_tags（領域標籤）
           │   └─ rationale（理由字串)
           └─ 輸出：PlannerManifest（JSON）
```

---

## 2.2 Planner 决策流程图（ASCII）

```
                    ┌──────────────────────┐
                    │  輸入：任務描述        │
                    │  "幫我建一個庫存系統..."│
                    └──────────┬───────────┘
                               │
                               ▼
                  ┌────────────────────────┐
                  │  S1: 需求確認           │
                  │  generate_questions()  │
                  │  ┌──────────────────┐  │
                  │  │ 檢查模糊點：       │  │
                  │  │  • 技術栈？       │  │
                  │  │ • 部署環境？       │  │
                  │  │ • 權限範圍？       │  │
                  │  └──────────────────┘  │
                  │           │              │
                  │           ▼              │
                  │  ┌──────────────────┐  │
                  │  │ 有待決問題？       │  │
                  │  └────────┬─────────┘  │
                  │           │            │
              ┌───┴───────────┴────────────┐│
              │YES                     NO ││
              ▼                            ▼│
    ┌─────────────────┐         ┌─────────────────┐
    │ stage=Confirming │         │ stage=Complete   │
    │ 輸出問題清單[]   │         │ 進入 S2 分析     │
    └─────────────────┘         └────────┬────────┘
                                          │
                                          ▼
                         ┌────────────────────────────┐
                         │  S2: 複雜度分析              │
                         │  ComplexityMetrics::        │
                         │  estimate_from_task()       │
                         │                            │
                         │  branches = count_keywords()│
                         │  diversity = match_domains()│
                         │  complexity = eval_struct() │
                         └─────────────┬──────────────┘
                                       │
                                       ▼
                         ┌────────────────────────────┐
                         │  S3: 分工決策                │
                         │  WorkMode::decide()         │
                         │                            │
                         │  Solo ┌─ branches ≤ 2       │
                         │        │ diversity ≤ 1      │
                         │        │ complexity ≤ 0.6   │
                         │  Fork └─ branches ≥ 3       │
                         │          │ diversity ≥ 2      │
                         │          │ complexity > 0.6   │
                         └─────────────┬──────────────┘
                                       │
                          ┌────────────┴────────────┐
                          │                         │
                          ▼                         ▼
               ┌──────────────────┐      ┌──────────────────┐
               │    Solo Mode     │      │    Fork Mode      │
               │                  │      │                  │
               │  ┌────────────┐  │      │  ┌────────────┐  │
               │  │ Planner   │  │      │  │ node-frontend  │
               │  │ Node (x1) │──│──▶   │  │ node-backend  │
               │  └────────────┘  │      │  │ node-database │
               │                  │      │  │ node-auth     │
               └──────────────────┘      │  └────────────┘  │
                                        │                  │
                                        │  依賴關係：        │
                                        │  fe→be→auth      │
                                        │  db (並行)       │
                                        └──────────────────┘
                                                   │
                                                   ▼
                                        ┌──────────────────┐
                                        │ PlannerManifest │
                                        │ (JSON output)   │
                                        └──────────────────┘
```

### 2.3 Dispatch Decision 閾值判定樹

```
ComplexityMetrics
      │
      ├── reasoning_branches ≥ 3 ──────────┐ YES → Fork
      │                                    │
      │  NO                                │
      ├── domain_diversity ≥ 2 ────────────┤ YES → Fork
      │                                    │
      │  NO                                │
      └── context_complexity > 0.6 ────────┤ YES → Fork
                                           │
                                           │ NO
                                           └──→ Solo
```

### 2.4 領域關鍵詞對照表

```
┌──────────┬──────────────────────────────────────────┬─────────┐
│ 領域     │ 關鍵詞                                    │ 輸出標籤 │
├──────────┼──────────────────────────────────────────┼─────────┤
│ 前端     │ 前端, ui, html, css, react, vue,         │ frontend│
│          │ 網站, web, 頁面, 電子商務, 商城, 電商    │         │
├──────────┼──────────────────────────────────────────┼─────────┤
│ 後端     │ 後端, api, server, backend, node, java   │ backend │
├──────────┼──────────────────────────────────────────┼─────────┤
│ 資料庫   │ 資料庫, sql, db, 儲存, 資料, mysql       │ database│
├──────────┼──────────────────────────────────────────┼─────────┤
│ 認證     │ 認證, auth, 登入, 權限, jwt, oauth       │ auth    │
├──────────┼──────────────────────────────────────────┼─────────┤
│ 部署     │ 部署, docker, ci, cd, k8s, server       │ devops  │
├──────────┼──────────────────────────────────────────┼─────────┤
│ 安全     │ 安全, 加密, 資安, xss, sql, injection   │ security│
├──────────┼──────────────────────────────────────────┼─────────┤
│ 效能     │ 效能, 優化, 快取, cache, load, scale     │ perf    │
├──────────┼──────────────────────────────────────────┼─────────┤
│ 測試     │ 測試, test, unit, integration, e2e, qa  │ testing │
└──────────┴──────────────────────────────────────────┴─────────┘
```

---

## 3. 核心決策邏輯

### 3.1 分工閾值（Dispatch Decision Rules）

| 指標 | 閾值 | Solo 條件 | Fork 條件 |
|------|------|---------|---------|
| `reasoning_branches` | 3 | ≤ 2 | ≥ 3 |
| `domain_diversity` | 2 | ≤ 1 | ≥ 2 |
| `context_complexity` | 0.6 | ≤ 0.6 | > 0.6 |

觸發 Fork 的條件（三者之一）：
- `branches >= 3` OR
- `diversity >= 2` OR
- `complexity > 0.6`

### 3.2 複雜度估算（Complexity Metrics）

```rust
struct ComplexityMetrics {
    reasoning_branches: u8,    // 0-5+（關鍵詞計數法）
    domain_diversity: u8,      // 0-4（領域關鍵詞匹配）
    context_complexity: f32,   // 0.0-1.0
}

// 估算公式：
// - reasoning_branches：檢測「分析、比較、整合、平行」等關鍵詞
// - domain_diversity：匹配「前端、後端、資料庫、認證、部署、安全」等領域
// - context_complexity：結構化程度（JSON/列表）+ 技術術語密度 + 長度評分
```

### 3.3 節點數估算

```rust
fn estimate_nodes(metrics: &ComplexityMetrics, mode: WorkMode) -> u8 {
    match mode {
        WorkMode::Solo => 1,
        WorkMode::Fork => (branches + diversity/2).clamp(2, 6),
    }
}
```

---

## 4. OS System 核心（v0.3.0 新增）

### 4.1 設計目標

將 Evolution OS 從「直線 Pipe」重構為「作業系統架構」：

| 架構類型 | 說明 |
|---------|------|
| **直線 Pipe（舊）** | Planner → Compiler → Executor，直接函式呼叫，無狀態管理 |
| **OS Kernel（新）** | 所有節點都是 Process，透過 `Kernel.syscall()` 互動，有獨立生命周期 |

### 4.2 核心元件

```
kernel/
├── mod.rs              # Kernel（本體）— syscall() 單一進場點
├── process.rs          # Process / ProcessState / Pid（含 Debug impl）
├── mailbox.rs          # Mailbox（FIFO 訊息佇列）
├── process_table.rs    # ProcessTable（index-based，PID=index）
├── scheduler.rs        # Scheduler（FIFO 排程，不遞迴 update）
├── syscall.rs          # SysCallKind（Spawn/Send/Receive/Wait/Exit）
└── system_process.rs  # SystemProcess trait（Node 包裝介面）
```

### 4.3 Process 狀態機

```
                    ┌───────┐
   spawn() ──▶     │ Ready │
                    └───┬───┘
                        │ scheduler 選中
                        ▼
                  ┌───────────┐
                  │ Running   │───▶ exit() ──▶ Zombie
                  └───────────┘
                        │
                   yield() ◀──┘
                        │
                        ▼
                  ┌───────────┐
                  │ Waiting   │◀──── wait(Pid)
                  └───────────┘
```

### 4.4 System Call 介面

所有行程透過 `Kernel::syscall()` 與核心互動：

```rust
pub enum SysCallKind {
    Spawn(Box<dyn SystemProcess>),    // 創建新行程
    Send { to: Pid, msg: String },   // 發送訊息
    Receive,                          // 接收訊息（blocking）
    Wait(Pid),                        // 等待指定行程結束
    Exit,                             // 行程結束
}
```

### 4.5 ProcessTable 設計

- **index = PID**：index 0 保留（無效 PID），PID 1 → `processes[1]`
- `spawn()`：分配新 PID，process 加入 Vec，状态设为 Ready
- `state()`：給定 PID，回傳 `ProcessState`

### 4.6 Scheduler 設計

- **FIFO 輪轉**：依序取出第一個 Ready 的 PID
- `sync_valid_pids()`：由 `Kernel.schedule()` 統一呼叫，移除已終止行程
- `next()`：**不遞迴呼叫** `update()`，只做 retain + pop_front

### 4.7 SystemProcess trait

行程包裝介面，讓 Node/Skill 可以作為 OS Process 運行：

```rust
pub trait SystemProcess: Send + Sync {
    fn name(&self) -> &str;
    fn tick(&mut self, kernel: &Kernel) -> ProcessResult;
    fn on_syscall(&mut self, call: SysCallKind, kernel: &Kernel) -> ProcessResult;
}
```

- `NodeProcess`：將現有 Node 封裝為行程
- `PlannerProcess`：將 Planner 封裝為行程

### 4.8 Planner → Kernel → Executor 整合

```
 PlannerManifest
      │
      ▼ spawn(PlannerProcess)
 ┌─────────┐     syscall(Spawn)      ┌─────────────┐
 │ Kernel  │ ──────────────────────▶ │ Planner PID=1│
 └─────────┘                        └─────────────┘
      │                                   │
      │  syscall(Send, node-frontend)     │
      ▼                                   │
 ┌─────────┐                              │
 │Executor │◀─── on_syscall ──────────────┘
 │ PID=2   │
 └─────────┘
```

### 4.9 測試結果

```
11 passed; 0 failed (kernel module)
106 passed; 3 failed (全體 — pre-existing failures, 與 kernel 無關)
```

---

## 5. 模組結構

### 5.1 代碼模組（src/）

| 模組 | 檔案 | 職責 |
|------|------|------|
| `planner` | `mod.rs` | 統一出口 |
| `planner::stages` | `stages.rs` | S1/S2/S3/Complete 枚舉 |
| `planner::decision` | `decision.rs` | 分工決策邏輯（ComplexityMetrics、WorkMode、DispatchDecision） |
| `planner::manifest` | `manifest.rs` |PlannerManifest 結構（JSON 輸出） |
| `evo` | `evo.rs` | Evolution OS 主類（整合所有子系統） |
| `node` | `mod.rs` + 附檔 | 節點抽象、MemoryGraph、NodeRegistry |
| `runtime` | `executor.rs` | 依賴排序執行、Executor |
| `model` | `dispatcher.rs` | AI 模型派遣（Ollama 支援） |
| `skill` | `filesystem.rs` | 技能實現（檔案、LLM、系統分析） |
| `kernel` | `mod.rs` + 附檔 | OS 系統核心：Process / Mailbox / Scheduler / ProcessTable / SysCall / SystemProcess |
| `storage` | `json_storage.rs` | Graph 持久化（JSON） |
| `chain` | `discovery.rs` | 呼叫鏈探索（葉→根 BFS） |

### 5.2 Planner CLI（src/bin/planner_cli.rs）

兩種使用模式：

```bash
# 直接模式
cargo run --bin planner_cli -- "幫我建一個庫存管理系統"

# 互動模式（多行輸入）
cargo run --bin planner_cli -- --interactive
```

輸出：
- 終端摘要（複雜度指標、分工模式、需求項目）
- 完整 JSON Manifest

### 5.3 測試結構

```
tests/
 └── planner_decision.rs    整合測試（6 cases）

src/planner/decision.rs     內建單元測試（4 cases）
```

整合測試覆蓋：
- 簡單任務 → Solo 模式
- 複雜任務（領域多樣）→ Fork 模式
- 領域標籤識別（database/frontend/backend/auth/devops/security/performance/testing）
- 節點數量估算範圍（1-6）
- 語境複雜度在合理範圍

---

## 6. 輸入輸出範例

### 6.1 輸入

```
幫我建一個庫存管理系統，要有前端、後端、資料庫、登入功能
```

### 6.2 PlannerManifest（JSON 輸出摘要）

```json
{
  "version": "0.1.0",
  "task": "幫我建一個庫存管理系統，要有前端、後端、資料庫、登入功能",
  "stage": "Confirming",
  "converged": false,
  "complexity": {
    "reasoning_branches": 1,
    "domain_diversity": 4,
    "context_complexity": 0.168
  },
  "work_mode": "Fork",
  "dispatch": {
    "mode": "Fork",
    "rationale": "branches=1, diversity=4, complexity=0.17 → Fork（多節點分工）",
    "estimated_nodes": 3,
    "domain_tags": ["database", "frontend", "backend", "auth"]
  },
  "requirements": [
    { "id": "req-1", "requirement": "前端介面", "priority": "Must", "dataType": "frontend" },
    { "id": "req-2", "requirement": "後端服務", "priority": "Must", "dataType": "backend" },
    { "id": "req-3", "requirement": "資料儲存", "priority": "Must", "dataType": "database" },
    { "id": "req-4", "requirement": "使用者認證", "priority": "Should", "dataType": "auth" }
  ],
  "questions": [
    { "id": "q-001", "question": "技術栈偏好？", "category": "Technical" },
    { "id": "q-002", "question": "部署環境偏好？", "category": "Technical" },
    { "id": "q-003", "question": "需要角色權限管理嗎？", "category": "Scope" }
  ],
  "estimated_nodes": [
    { "id": "node-frontend", "role": "前端工程師", "handles": [...], "depends_on": [] },
    { "id": "node-backend", "role": "後端工程師", "handles": [...], "depends_on": ["node-frontend"] },
    { "id": "node-database", "role": "資料庫工程師", "handles": [...], "depends_on": [] },
    { "id": "node-auth", "role": "安全工程師", "handles": [...], "depends_on": ["node-backend"] }
  ]
}
```

---

## 7. 期末展示大綱

### 7.1 Demo 項目（終端展示）

```bash
# 1. Planner 分工決策展示
cargo run --bin planner_cli -- "幫我建一個庫存管理系統"

# 2. 互動模式
cargo run --bin planner_cli -- --interactive

# 3. 測試通過
cargo test --test planner_decision

# 4. 同時演示 Solo vs Fork
cargo run --bin planner_cli -- "寫一個計數器網頁"   # Solo
cargo run --bin planner_cli -- "幫我建一個庫存管理系統"  # Fork
```

### 7.2 預期展現的能力

1. **可讀的分析報告**：終端直接顯示分工人推薦理由
2. **結構化 JSON 輸出**：可直接交給下層系統執行
3. **清晰的問題清單**：告知用户還需要確認什麼
4. **測試覆蓋驗證**：88 個單元測試 + 6 個整合測試

---

## 8. 版本歷程

| 版本 | 日期 | 內容 |
|------|------|------|
| v0.3.0 | 2026-05-28 | OS System 核心：kernel module（Process/Mailbox/Scheduler/ProcessTable/SysCall/SystemProcess），sync Rust，11 tests passed |
| v0.2.0 | 2026-05-27 | 期末文件：SPEC.md 規格書 + REPORT.md 報告 + CHANGELOG.md + VERSION_CONTROL.md |
| v0.1.0 | 2026-05-27 | Planner 核心：stages + decision + manifest + CLI + 整合測試 |

---

## 9. 設計原則與擴展方向

### 8.1 設計原則

1. **問題未確認不實作**: S1 强制產出問題清單，防止方向錯誤
2. **資訊重構複雜度驅動分工**: 分工依據是「產出最佳解答所需的推理複雜度」，不是「任務數量」
3. **邏輯上的遞迴**: Planner 的優化輸出 → 更好的下次輸入結構（不是 literal self-feeding）
4. **可驗證的決策**: 每個分工決策都有可讀的 `rationale`，用户可理解和糾正

### 8.2 v0.3 預期工作

- **Compiler 整合**: PlannerManifest → 實際 Node 調度執行
- **Visible Decision**: 完整推斷過程輸出（為何這個任務複雜、如何計算每個指標）
- **Context Injection**: 每次對話注入完整 graph 狀態到 system prompt（gemma4 沒有持久記憶）
- **多語言支援**: 任務描述支援英文，關鍵詞也支援英文領域檢測

---

## 10. 技術規格

- **語言**: Rust（stable, 2021 edition）
- **Dependencies**:
  - `serde` / `serde_json` — 序列化
  - `chrono` — timestamp
  - `tokio` — async runtime（預設 disable，future use）
  - `anyhow` / `thiserror` — error handling
- **Build**: `cargo build --bin planner_cli`
- **Test**: `cargo test`（88 unit + 6 integration）
- **Format**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Platform**: macOS/Linux（不支援 Windows 直接編譯）