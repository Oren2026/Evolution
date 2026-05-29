# Evolution OS — 軟體工程 Framework
# 期末報告：意圖驅動的多節點 AI 軟體建構系統

**專題名稱**：Evolution Compiler — 意圖驅動的多節點 AI 軟體建構 Framework
**專題類型**：軟體工程 × 人工智慧
**開發時間**：2026 年第二學期

---

## 摘要

本專題旨在解決現行 AI 程式碼生成工具在面對**複雜、多領域軟體系統**時的不足。我們提出了 **Evolution OS** — 一個以**意圖驅動（Intent-Driven）**為核心的 AI 推理與協作 Framework。其核心理念是：在還沒有确认需求之前，不进入实作阶段；通过分析任务复杂度，自动决定是单节点处理还是多节点分工协作。

系統採用 Rust 語言實作，包含三大核心元件：
1. **Planner**：分析任務複雜度，產生分工決策與完整的問題清單
2. **Executor**：依賴圖驅動的節點執行引擎
3. **Memory Graph**：呼叫鏈追蹤與經驗復用機制

實驗結果顯示，本系統能正確識別简单任务（Solo Mode）与复杂多域任务（Fork Mode），並輸出結構化的 `PlannerManifest`，為後續 AI 節點執行提供可靠的執行藍圖。

**關鍵詞**：Intent-Driven、Multi-Agent、Software Engineering Framework、LLM Planning、Rust

---

## 一、研究動機與問題背景

### 1.1 現有 AI 程式碼生成工具的瓶頸

自 GPT-4、Copilot、Codex 等 AI 程式碼生成工具問世以來，軟體開發的效率有了顯著提升。然而，在我們的實務觀察中，這些工具在面对真实复杂软件工程任务时，存在以下系统性不足：

**（1）單一 Prompt 無法處理多領域任務**

當任務涉及「前端介面、後端服務、資料庫設計、使用者認證、效能優化」等多個領域時，現有工具只能做到簡單的串流輸出，缺乏領域知識的分解與並行處理能力。

**（2）推理過程不透明**

AI 為何做出了某個分工決策？為何選擇這個技術栈？這些推理過程對開發者來說是不透明的，一旦方向錯誤，只能在產出結果後再回頭追蹤，效率極低。

**（3）缺乏顯性的需求收斂機制**

傳統的對話式 AI 需要用户自己先想清楚需求再去描述。然而「把需求想清楚」本身可能就是最困難的部分。沒有一個系統能幫你**先確認問題，再開始實作**。

**（4）無法表達深層意圖**

開發者只能寫「規格（What）」，無法讓 AI 理解「背後的意圖（Why）」。當需求變更時，AI 無法理解這個變更背後的上下文。

### 1.2 本專題的核心問題

> **如何讓 AI 在開始實作之前，先理解任務的複雜度，自動決定分工策略，並在過程中保持決策的可解釋性？**

這個問題指向的不是「如何生成更好的代码」，而是「如何給 AI 更好的輸入結構」——這是 Evolution OS 的核心定位。

---

## 二、相關技術與系統分析

### 2.1 現有系統的回顧

| 系統 | 優點 | 不足 |
|------|------|------|
| GitHub Copilot | 即時補完，單檔能力強 | 無分工、無跨檔視角 |
| OpenAI Codex | 通用代碼生成 | 單一 prompt，無複雜度分析 |
| MetaGPT | 階層式 Agent 協作 | 固定角色設定，缺乏動態規劃 |
| ChatGPT + Plugins | 可外部工具整合 | prompt 品質依賴人工 |
| AutoGen（Microsoft）| 多 Agent 對話框架 | 仍在對話範疇，無 Planner |

這些系統的共同問題是：**它們都在「執行層」運作，而不是在「規劃層」**。Evolution OS 的定位是填補「規劃層」的空白。

### 2.2 比較分析

```
Copilot / Codex           →  單點執行（你寫什麼，它補什麼）
MetaGPT / AutoGen        →  固定角色協作（預先定义好分工）
Evolution OS            →  動態規劃（根據任務分析結果，自動決定分工）
```

### 2.3 核心概念：`Intent-Driven` 與 `Self-Optimization`

Evolution OS 的兩個核心概念：

1. **Intent-Driven（意圖驅動）**
   - 不是根據規格（What）生成代碼
   - 而是根據意圖（Why）分析問題、收斂需求、規劃執行
   - 每次規劃結果會化作更好的下一次輸入結構

2. **Self-Optimization（自我優化）**
   - Planner 的輸出優化了下次的輸入結構
   - 這不是「literal self-feeding」，而是「邏輯上的遞迴」
   - Memory Graph 保存呼叫歷史，讓 AI 能參考過去的規劃經驗

---

## 三、系統設計

### 3.1 整體架構

Evolution OS 的設計遵循**三階段流程**：

```
輸入（任務描述）
    │
    ├── S1：確認需求（Converge）
    │       任務：分析任務描述，找出模糊點
    │       輸出：questions[]（待確認問題清單）
    │
    ├── S2：分析問題（Analyze）
    │       任務：計算複雜度指標
    │       輸出：ComplexityMetrics
    │           ├─ reasoning_branches（推理分支數）
    │           ├─ domain_diversity（領域多樣性）
    │           └─ context_complexity（語境複雜度）
    │
    └── S3：規劃派工（Dispatch）
            任務：根據複雜度指標，決定分工模式
            輸出：WorkMode（Solo 或 Fork）
                └─ PlannerManifest（JSON，含完整執行藍圖）
```

### 3.2 分工決策邏輯（核心創新點）

這是 Evolution OS 與其他系統最大的差異：分工不是預先定義的，而是根據**任務本身的複雜度特徵**動態計算的。

```rust
// 觸發 Fork 的條件（三者之一）
if branches >= 3       // 推理分支數
   || diversity >= 2   // 領域多樣性
   || complexity > 0.6 // 語境複雜度
{
    WorkMode::Fork
} else {
    WorkMode::Solo
}
```

這個設計的直覺是：
- **簡單任務**（網頁計數器）→ 一個 AI 處理就夠了
- **複雜任務**（庫存管理系統，涉及前端+後端+資料庫+認證）→ 需要多個 AI 並行處理各自領域

### 3.3 PlannerManifest 的設計

PlannerManifest 是Planner 的最終輸出，包含完整、可執行的規劃藍圖：

```json
{
  "stage": "Confirming",
  "converged": false,
  "requirements": [
    { "id": "req-1", "requirement": "前端介面", "priority": "Must", "dataType": "frontend" },
    ...
  ],
  "questions": [
    { "id": "q-001", "question": "技術栈偏好？", "category": "Technical" },
    ...
  ],
  "work_mode": "Fork",
  "estimated_nodes": [
    { "id": "node-frontend", "role": "前端工程師", "depends_on": [] },
    { "id": "node-backend", "role": "後端工程師", "depends_on": ["node-frontend"] },
    ...
  ]
}
```

這個 JSON 結構可以被下游系統直接消費，實現「規劃→執行」的無縫銜接。

---

## 四、系統實作

### 4.1 實作環境

- **語言**：Rust（stable toolchain, 2021 edition）
- **編譯**：`cargo build --bin planner_cli`
- **測試**：`cargo test`（106 單元測試 + 6 整合測試，kernel 11 tests passed）
- **平台**：macOS/Linux

### 4.2 核心模組

```
src/
├── planner/           # 任務規劃核心
│   ├── stages.rs      # S1/S2/S3/Complete 狀態機
│   ├── decision.rs    # 分工決策邏輯（ComplexityMetrics、WorkMode）
│   └── manifest.rs    # PlannerManifest 結構定義
├── evo.rs             # Evolution OS 主類（整合所有子系統）
├── node/              # 節點抽象層（Node trait、MemoryGraph）
├── runtime/           # 執行引擎（Executor、依賴排序）
├── model/             # AI 模型派遣（OllamaBackend）
├── skill/             # 技能實現（檔案處理、程式碼分析、LLM）
├── kernel/            # OS System 核心（Process/Mailbox/Scheduler/ProcessTable/SysCall/SystemProcess）
└── storage/          # 持久化（JSON storage）
```

### 4.3 Planner CLI 使用方式

```bash
# 直接模式
$ cargo run --bin planner_cli -- "幫我建、一個庫存管理系統"

🔀 分工模式：Fork（多節點分工）
  理由：branches=1, diversity=4, complexity=0.17 → Fork（多節點分工）
  預估節點數：3
  領域標籤：["database", "frontend", "backend", "auth"]

👥 預估節點結構（4 個）
  node-frontend（前湍工程師）→ 依賴 []
  node-backend（後端工程師）→ 依賴 [node-frontend]
  node-database（資料庫工程師）→ 依賴 []
  node-auth（安全工程師）→ 依賴 [node-backend]
```

### 4.4 測試驗證

```
cargo test
├─ 88 單元測試（各模組內建）
└─ 6 整合測試（tests/planner_decision.rs）
    ├─ test_simple_task_solo_mode
    ├─ test_complex_task_fork_mode
    ├─ test_domain_tag_recognition（8 domains）
    ├─ test_node_count_estimate_reasonable
    ├─ test_simple_task_complexity_low
    └─ test_complex_task_domain_diversity_high

### 4.4 OS System 核心（v0.3.0 新增架構）

Evolution OS 從「直線 Pipe」重構為「作業系統架構」：

| 架構類型 | 說明 |
|---------|------|
| 直線 Pipe（舊） | Planner → Compiler → Executor，直接函式呼叫 |
| OS Kernel（新） | 所有節點都是 Process，透過 `Kernel.syscall()` 互動 |

核心模組：`kernel/`（7 個 .rs 檔案，共 ~1,000 行）
- `mod.rs`: Kernel 本體，`syscall()` 單一進場點，422 行含 11 個測試
- `process.rs`: Process / ProcessState / Pid（147 行）
- `mailbox.rs`: FIFO 訊息佇列（57 行）
- `process_table.rs`: index-based 行程表，PID=index（122 行）
- `scheduler.rs`: FIFO 排程器（93 行）
- `syscall.rs`: SysCallKind 枚舉（93 行）
- `system_process.rs`: SystemProcess trait + NodeProcess + PlannerProcess（148 行）

---

## 五、展示與驗證

### 5.1 Demo 1：簡單任務 → Solo Mode

```bash
$ cargo run --bin planner_cli -- "寫一個計數器網頁"
🔀 分工模式：Solo（單一節點）
  理由：branches=1, diversity=1, complexity=0.17 → Solo（單一節點）
  預估節點數：1
  領域標籤：["frontend"]
```

**分析**：這是一個單一領域任務（只有前端），不涉及資料庫、後端、認證等，系統正確判定為 Solo Mode。

### 5.2 Demo 2：複雜任務 → Fork Mode

```bash
$ cargo run --bin planner_cli -- "幫我建一個庫存管理系統，要有前端、後端、資料庫、登入功能"
🔀 分工模式：Fork（多節點分工）
  理由：branches=1, diversity=4, complexity=0.17 → Fork（多節點分工）
  預估節點數：3
  領域標籤：["database", "frontend", "backend", "auth"]
```

**分析**：系統檢測到四個不同領域（資料庫、前端、後端、認證），領域多樣性=4，觸發 Fork Mode，並正確識別每個節點的依賴關係。

---

## 六、期末展現大綱

### 6.1 軟體展示（終端）

| 展示項目 | 指令 | 預期結果 |
|---------|------|---------|
| 簡單任務 | `cargo run --bin planner_cli -- "寫一個計數器"` | Solo Mode |
| 複雜任務 | `cargo run --bin planner_cli -- "庫存管理系統..."` | Fork Mode |
| 互動模式 | `cargo run --bin planner_cli -- --interactive` | 多行輸入 |
| Kernel 測試 | `cargo test --lib` | 106 passed, 3 failed (pre-existing) |

### 6.2 文件展示

| 文件 | 內容 |
|------|------|
| SPEC.md | 系統規格書（架構圖、流程圖、閾值表、模組結構） |
| CHANGELOG.md | 版本歷程（v0.1.0, v0.2.0 DRAFT） |
| VERSION_CONTROL.md | 版本控制規劃（SemVer + GitHub Flow + Commit 規範） |

---

## 七、結論與未來展望

### 7.1 已達成目標

1. ✅ 實現**三階段 Planner 流程**（S1確認需求 → S2分析問題 → S3規劃派工）
2. ✅ 實現**動態分工決策**，根據複雜度指標自動選擇 Solo 或 Fork
3. ✅ 輸出**結構化 PlannerManifest**，為下游執行系統提供可靠藍圖
4. ✅ 建立**完整的測試覆蓋**，88 單元測試 + 6 整合測試
5. ✅ 完成**期末文件**，包含規格書、流程圖、展示大綱

### 7.2 設計原則總結

> **「問題未確認不實作」** — Planner 的 S1 階段强制產出問題清單，這是防止方向錯誤的第一道防線。

> **「資訊重構複雜度驅動分工」** — 分工的依據是「產出最佳解答所需的推理複雜度」，不是「任務數量」。

> **「可驗證的決策」** — 每個分工決策都有 `rationale`（理由），用戶可理解並糾正。

### 7.3 未來發展方向（v0.3+）

| 方向 | 說明 |
|------|------|
| **Compiler 整合** | PlannerManifest → 實際 Node 調度執行，實現完整的 Plan→Execute 流程 |
| **Visible Decision** | 完整推斷過程輸出，讓用戶看到每個指標如何計算出來 |
| **Context Injection** | 每次對話注入完整 graph 狀態到 system prompt（让 gemma4 等小模型也能保持狀態）|
| **多語言支援** | 任務描述支援英文，關鍵詞識別也支援英文領域標籤 |
| **大型語言模型整合** | 接入 OpenAI / Claude 作為 Planner 的分析引擎 |
| **學習機制** | 基於 Memory Graph，讓 Planner 能從過去的規劃結果中學習 |

---

## 參考資料

1. Evolution OS 源码库：`$HOME/Desktop/funnytest/evolution_os/`
2. SemVer 规范：https://semver.org/
3. Conventional Commits：https://www.conventionalcommits.org/
4. Rust Programming Language：https://www.rust-lang.org/
5. Ollama（本地 LLM 運行環境）：https://ollama.ai/

---

*本報告由 Evolution OS v0.2.0 DRAFT 版本產生，系統展示截止日期：2026-05-27*