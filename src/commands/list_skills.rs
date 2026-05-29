//! list_skills — 列出可用技能

pub fn list_skills() {
    println!();
    println!(" Evolution OS — 可用技能");
    println!("═{}", "═".repeat(39));
    println!();

    // 從 skill registry 讀取
    // 目前是靜態的，等下可以從 SystemProcess 枚舉

    println!("技能分類：");
    println!();
    println!("  [filesystem]  檔案系統操作");
    println!("    • read_dir     讀取目錄");
    println!("    • file_stats   檔案統計");
    println!("    • path_expand  路徑展開");
    println!();
    println!("  [analysis]    程式碼分析");
    println!("    • project_analyzer  專案分析");
    println!("    • count_lines       行數統計");
    println!();
    println!("  [llm]         語言模型");
    println!("    • gemma4       推理引擎");
    println!("    • mistral      備用引擎");
    println!();
    println!("  [runtime]     執行引擎");
    println!("    • graph_executor  節點圖執行");
    println!("    • planner         任務規劃");
    println!();
    println!("═{}", "═".repeat(39));
    println!("共 3 個分類，11 個技能");
    println!();
}