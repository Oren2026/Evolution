const pptxgen = require('pptxgenjs');
const pptx = new pptxgen();

// 配色
const C = { dark: '1A1A2E', blue: '16213E', teal: '0F3460', accent: '00ADB5', white: 'EEEEEE', gray: 'AAAAAA' };

// 投影片內容
const slides = [
  {
    title: 'Evolution OS',
    subtitle: '意圖驅動的多節點 AI 軟體建構 Framework',
    type: 'cover',
    content: ['軟體工程 × 人工智慧', '2026 年第二學期 期末報告']
  },
  {
    title: '研究動機',
    subtitle: '現有 AI 程式碼生成工具的瓶頸',
    type: 'content',
    content: [
      '▸ 單一 Prompt 無法處理多領域任務（前端/後端/資料庫）',
      '▸ 推理過程不透明，方向錯誤只能事後追蹤',
      '▸ 缺乏顯性的需求收斂機制',
      '▸ 只能表達「規格（What）」，無法理解「意圖（Why）」'
    ]
  },
  {
    title: '核心問題',
    subtitle: '如何讓 AI 在實作前先理解任務複雜度？',
    type: 'problem',
    content: [
      '「如何讓 AI 在開始實作之前，先理解任務的複雜度，自動決定分工策略，並在過程中保持決策的可解釋性？」'
    ]
  },
  {
    title: '系統架構',
    subtitle: 'Planner → Compiler → Executor',
    type: 'architecture',
    content: [
      '輸入（任務描述）',
      '  └─ Planner（三階段决策）',
      '       ├─ S1：確認需求 → 問題清單',
      '       ├─ S2：分析問題 → 複雜度指標',
      '       └─ S3：規劃派工 → Solo / Fork',
      '  └─ Compiler（Manifest → NodeGraph）',
      '       ├─ ExecutionGraph：拓撲排序 + 循環偵測',
      '       └─ NodeFactory：角色系統提示詞生成',
      '  └─ Executor（依賴圖驅動執行）',
      '       └─ GraphExecutor：按 tier 順序執行'
    ]
  },
  {
    title: 'Planner 分工決策',
    subtitle: 'Solo vs Fork 模式判定',
    type: 'decision',
    content: [
      '閾值判斷樹：',
      '  branches ≤ 2 且 diversity ≤ 1 且 complexity ≤ 0.6',
      '  → Solo（單節點處理）',
      '',
      '  否則 → Fork（多節點分工）',
      '  每個子節點有 role（planner/analyst/architect/qa）',
      '  以及 expected_output'
    ]
  },
  {
    title: 'Compiler 轉譯流程',
    subtitle: 'PlannerManifest → 可執行節點圖',
    type: 'flow',
    content: [
      '1. ExecutionGraph::from_manifest()',
      '   → 建立節點圖，計算拓撲分層（tier 0/1/2...）',
      '',
      '2. NodeFactory::create_nodes()',
      '   → 根據 EstimatedNode 的 role 生成系統提示詞',
      '   → 建立 LLMNode（Planner/Analyst/Architect/QA）',
      '',
      '3. 循環依賴偵測',
      '   → 若有循環依賴，回傳 GraphError::CyclicDependency'
    ]
  },
  {
    title: 'Executor 執行流程',
    subtitle: 'Tier-based 節點圖執行',
    type: 'flow',
    content: [
      '1. execution_order()',
      '   → 按 tier 分組，同一 tier 可並行執行',
      '',
      '2. 依序執行每個 tier 的節點',
      '   → 每個節點接收上游輸出作為 context',
      '   → 記錄輸出到 tier_outputs，供下游使用',
      '',
      '3. graph.hit(node_id)',
      '   → 更新節點命中計數（用於熱度追蹤）'
    ]
  },
  {
    title: 'OS System 核心',
    subtitle: '從直線 Pipe 重構為作業系統架構',
    type: 'architecture',
    content: [
      'v0.3.0 新增：kernel/（sync Rust，tokio-less）',
      '',
      'kernel/mod.rs       — Kernel 本體，syscall() 單一進場點（422行）',
      'kernel/process.rs   — Process / ProcessState / Pid（147行）',
      'kernel/mailbox.rs   — FIFO 訊息佇列（57行）',
      'kernel/process_table.rs — index-based 行程表（122行）',
      'kernel/scheduler.rs — FIFO 排程器（93行）',
      'kernel/syscall.rs   — SysCallKind（93行）',
      'kernel/system_process.rs — SystemProcess trait（148行）',
      '',
      'ProcessTable: index 0 保留（無效 PID），PID=index',
      'Scheduler: FIFO，不遞迴 update()，sync_valid_pids() 統一維護'
    ]
  },
  {
    title: '模組結構',
    subtitle: 'v0.1.0 ~ v0.4.0 演進歷程',
    type: 'modules',
    content: [
      'v0.1.0 — Planner（決策邏輯、分工閾值）',
      'v0.2.0 — 規格文件 + 期末投影片',
      'v0.3.0 — OS System 核心（Kernel module）+ Compiler 整合',
      'v0.4.0 — Planner→Kernel→Executor 端對端流程',
      '',
      '核心模組：',
      'src/planner/   — decision.rs, stages.rs, manifest.rs',
      'src/kernel/    — mod.rs, process.rs, mailbox.rs, scheduler.rs, syscall.rs, system_process.rs',
      'src/runtime/   — executor.rs, graph_executor.rs, dispatcher.rs',
      'src/node/      — mod.rs, dyn_skill_node.rs, skill_node.rs',
      'src/skill/     — registry.rs, llm.rs, analysis.rs, filesystem.rs'
    ]
  },
  {
    title: 'Demo 展示',
    subtitle: '端到端執行流程演示',
    type: 'demo',
    content: [
      '（待實作 — 需要端到端測試腳本）'
    ]
  },
  {
    title: '結論與未來展望',
    subtitle: '從規劃到執行的完整流程',
    type: 'conclusion',
    content: [
      '已實現：Planner（三階段分工）+ OS Kernel（Process/Mailbox/Scheduler）+ Compiler（藍圖轉譯）+ Executor（依賴驅動）',
      '',
      '未來工作：',
      '▸ v0.4.0 — Planner→Kernel→Executor 端對端流程（Node 包裝、Executor 行程）',
      '▸ v0.5.0 — Storage（圖的持久化，load/save）',
      '▸ v0.6.0 — Memory Graph（呼叫鏈追蹤與經驗復用）',
      '',
      '核心定位：不只是 Code Generator，是 AI Reasoning/Collaboration Tool'
    ]
  }
];

// 投影片標題字型
const titleFont = 'Arial';
const contentFont = 'Arial';

// 通用投影片背景
function addBg(slide) {
  slide.addShape('rect', { x: 0, y: 0, w: '100%', h: '100%', fill: { color: C.dark } });
}

// 通用頁尾
function addFooter(slide, pageNum, total) {
  slide.addText(`${pageNum} / ${total}`, {
    x: 0, y: 6.8, w: '100%', h: 0.3,
    fontSize: 10, color: C.gray, align: 'right'
  });
}

// 通用分隔線
function addDivider(slide) {
  slide.addShape('rect', { x: 0.5, y: 1.6, w: 12, h: 0.02, fill: { color: C.accent } });
}

// 通用子標題
function addSubtitle(slide, subtitle) {
  slide.addText(subtitle, {
    x: 0.5, y: 1.7, w: 12, h: 0.5,
    fontSize: 16, color: C.accent, fontFace: contentFont
  });
}

// 通用內容
function addContent(slide, contentArr) {
  const text = contentArr.map((line, i) => ({
    text: line,
    options: { breakLine: i < contentArr.length - 1 }
  }));
  slide.addText(text, {
    x: 0.5, y: 2.4, w: 12, h: 3.8,
    fontSize: 14, color: C.white, fontFace: contentFont,
    paraSpaceAfter: 6
  });
}

// 通用標題
function addTitle(slide, title) {
  slide.addText(title, {
    x: 0.5, y: 0.5, w: 12, h: 1,
    fontSize: 36, bold: true, color: C.white, fontFace: titleFont
  });
}

// ========== 建立投影片 ==========

slides.forEach((s, i) => {
  const pageNum = i + 1;
  const total = slides.length;

  const slide = pptx.addSlide();
  addBg(slide);

  if (s.type === 'cover') {
    // 封面
    slide.addText(s.title, {
      x: 0, y: 2, w: '100%', h: 1.2,
      fontSize: 52, bold: true, color: C.white, fontFace: titleFont, align: 'center'
    });
    slide.addText(s.subtitle, {
      x: 0, y: 3.3, w: '100%', h: 0.8,
      fontSize: 20, color: C.accent, fontFace: contentFont, align: 'center'
    });
    slide.addShape('rect', { x: 4, y: 4.2, w: 5, h: 0.04, fill: { color: C.accent } });
    const meta = s.content.join('  ·  ');
    slide.addText(meta, {
      x: 0, y: 4.5, w: '100%', h: 0.5,
      fontSize: 14, color: C.gray, fontFace: contentFont, align: 'center'
    });
  } else if (s.type === 'problem') {
    // 問題（引用格式）
    addTitle(slide, s.title);
    addDivider(slide);
    addSubtitle(slide, s.subtitle);
    slide.addShape('rect', { x: 1, y: 2.5, w: 0.08, h: 2, fill: { color: C.accent } });
    slide.addText(s.content[0], {
      x: 1.3, y: 2.5, w: 10, h: 2,
      fontSize: 16, color: C.white, fontFace: contentFont, italic: true
    });
  } else if (s.type === 'architecture' || s.type === 'flow') {
    // 架構/流程（等寬字）
    addTitle(slide, s.title);
    addDivider(slide);
    addSubtitle(slide, s.subtitle);
    slide.addText(
      s.content.map(line => ({ text: line, options: { breakLine: true } })),
      {
        x: 0.5, y: 2.4, w: 12, h: 4,
        fontSize: 13, color: C.white, fontFace: 'Courier New',
        paraSpaceAfter: 4
      }
    );
  } else if (s.type === 'demo') {
    // Demo（placeholder）
    addTitle(slide, s.title);
    addDivider(slide);
    addSubtitle(slide, s.subtitle);
    slide.addShape('rect', { x: 3, y: 3, w: 7, h: 2, fill: { color: C.teal } });
    slide.addText('（待實作）', {
      x: 3, y: 3.5, w: 7, h: 1,
      fontSize: 18, color: C.gray, fontFace: contentFont, align: 'center'
    });
  } else {
    // 一般內容
    addTitle(slide, s.title);
    addDivider(slide);
    addSubtitle(slide, s.subtitle);
    addContent(slide, s.content);
  }

  addFooter(slide, pageNum, total);
});

// 儲存
pptx.writeFile({ fileName: 'Evolution_OS_期末報告.pptx' })
  .then(() => console.log('OK'))
  .catch(err => console.error('Error:', err));