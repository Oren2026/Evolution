# Evolution OS — Conversation History

## 2026-05-28 | LLM 串接 + Streaming 安裝

---

### 對話一：用戶確認要做 LLM 串接

**User**: 我們剛剛不是討論 llama3 嗎

**Assistant**: 抱歉，說錯了。我重新確認一下之前確立的模型狀態：
- **llama3** ✅ 正常
- **mistral:7b-instruct** ✅ 正常
- **gemma4:e4b** ❌ JSON 輸出有 bug（done_reason=length）
- **gemma4:e2b** ❌ 中文 prompt 回空白

目前 Planner 的實作（`decision.rs` / `manifest.rs`）是純規則，沒有真的 call LLM。要串接的話，選 **llama3**（本地 ollama）還是 **mistral**？

---

**User**: 可以嘗試串接，並把安裝llama3的部分也包裝在安裝之中

**Assistant**: 好，確認方向：
1. Planner 的 `ComplexityMetrics` 接入 llama3（分析任務複雜度）
2. `DispatchDecision` 的 domain_tags 由 llama3 提取
3. `--install` 時自動 `ollama pull llama3`（streaming 進度）

確認沒問題就開始做。

---

**User**: 現在做

---

### 對話二：實作 + 發現 CLI arg bug

**Assistant**: （開始實作...）

發現問題：`./target/release/planner_cli --llm -- "幫我建一個庫存管理系統"` 中 `--` 分隔符被錯誤包含在 task 中（變成 `task: "-- 幫我建..."`）。

修復 `planner_cli.rs` 的 arg parsing：
- `use_llm = true` 時，若 `args[2] == "--"` 则从 3 开始，否则从 2 开始

---

### 對話三：LLM 串接完成，展示結果

**Assistant**: LLM 串接完成。

**已實作功能（`--llm` 模式）**

| 功能 | 說明 |
|------|------|
| **Complexity 分析** | llama3 分析任務的 reasoning_branches、domain_diversity、context_complexity |
| **領域標籤提取** | llama3 從任務描述辨識 frontend/backend/database/auth 等領域 |
| **LLM-driven 決策** | complexity 0.7 → Fork 模式，4 nodes（frontend/backend/database/auth）|

**使用方式**
```bash
cargo build --release --features llm
./target/release/planner_cli --llm -- "幫我建一個庫存管理系統"
```

**已 push**：`78ce772 feat: full LLM integration for planner`

---

**User**: 給我GitHub這部分的網址

**Assistant**: https://github.com/Oren2026/funnytest
Evolution OS 在 `evolution_os/` 目錄下。

---

### 對話四：--install 也要 streaming 下載進度

**User**: 然後整理我們的對話紀錄，再新增一個conversation history

**Assistant**: 至於 LLM 串接的部分，要真正串接 llama3 的話，需要在 decision.rs 或新建一個 LLM 驅動的規劃節點。這個你確定要做嗎？

---

**User**: 要

**Assistant**: 實作 streaming 安裝。`pull_llama3()` 現在以 streaming 方式顯示下載進度：
- 雙執行緒分別讀取 stdout/stderr，即時列印 progress lines
- 4-5GB 模型下載時可看到進度條
- fallback: pull 返回非零時檢查 model 是否已存在

**已 push**：`87f54fe feat(installer): streaming ollama pull with real-time progress output`

---

## Commit Log

| Commit | 說明 |
|--------|------|
| `78ce772` | feat: full LLM integration for planner — complexity + domain_tags via llama3 |
| `87f54fe` | feat(installer): streaming ollama pull with real-time progress output |

---

## 技術產出

| 功能 | 檔案 | 實作 |
|------|------|------|
| LLM complexity 分析 | `src/planner/decision.rs` — `ComplexityMetrics::estimate_with_llm()` | llama3 → {branches, diversity, complexity} |
| LLM domain tags | `src/planner/manifest.rs` — `extract_domain_tags_llm()` | llama3 → ["frontend", "backend", "database", "auth"] |
| LLM-driven 決策 | `src/planner/decision.rs` — `DispatchDecision::from_metrics()` | 接收 LLM 分析結果，WorkMode::decide() |
| Streaming model 下載 | `src/system/installer.rs` — `pull_llama3()` | 雙執行緒 streaming stdout/stderr |
| CLI arg 修復 | `src/bin/planner_cli.rs` | 正確處理 `--llm -- "task"` 格式 |

---

## Repo

https://github.com/Oren2026/funnytest