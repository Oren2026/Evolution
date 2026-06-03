//! test_record — 測試環境隔離與記錄系統
//!
//! 每次測試後：
//!   1. 建立 ~/.evolution/tests/YYYY-MM-DD-HHMMSS/ 目錄
//!   2. 寫入 input.txt / manifest.ev / result.json / timing.json
//!   3. 環境清空（只保留記錄檔）

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// 測試記錄
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestRecord {
    /// 測試時間（ISO 8601）
    pub timestamp: String,
    /// 任務描述
    pub task: String,
    /// 產出 manifest 摘要
    pub manifest_summary: ManifestSummary,
    /// 執行時間（毫秒）
    pub duration_ms: u64,
    /// 使用的模型
    pub model: String,
    /// 測試結果：success / failed
    pub result: String,
    /// 錯誤訊息（如果有）
    pub error: Option<String>,
}

/// Manifest 摘要（AI 可讀格式）
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ManifestSummary {
    pub work_mode: String,
    pub reasoning_branches: u8,
    pub domain_diversity: u8,
    pub context_complexity: f32,
    pub estimated_nodes: usize,
    pub domain_tags: Vec<String>,
}

/// 開始測試計時
pub fn start_test() -> Instant {
    Instant::now()
}

/// 記錄測試結果
pub fn record_test(
    task: &str,
    manifest_json: &str,
    duration_ms: u64,
    model: &str,
    success: bool,
    error_msg: Option<&str>,
) -> PathBuf {
    let root = evolution_root();
    let timestamp = chrono::Utc::now();
    let dir_name = timestamp.format("%Y-%m-%d-%H%M%S").to_string();
    let test_dir = root.join("tests").join(&dir_name);

    // 建立測試目錄
    fs::create_dir_all(&test_dir).ok();

    // 解析 manifest 摘要
    let summary = parse_manifest_summary(manifest_json);

    let record = TestRecord {
        timestamp: timestamp.to_rfc3339(),
        task: task.to_string(),
        manifest_summary: summary,
        duration_ms,
        model: model.to_string(),
        result: if success { "success".to_string() } else { "failed".to_string() },
        error: error_msg.map(|s| s.to_string()),
    };

    // 寫入 result.json（AI 可讀）
    let result_path = test_dir.join("result.json");
    fs::write(&result_path, serde_json::to_string_pretty(&record).unwrap()).ok();

    // 寫入 input.txt
    let input_path = test_dir.join("input.txt");
    fs::write(&input_path, task).ok();

    // 寫入 manifest.ev
    let manifest_path = test_dir.join("manifest.ev");
    fs::write(&manifest_path, manifest_json).ok();

    // 寫入 timing.json
    let timing = serde_json::json!({
        "started_at": timestamp.to_rfc3339(),
        "duration_ms": duration_ms,
        "model": model
    });
    let timing_path = test_dir.join("timing.json");
    fs::write(&timing_path, serde_json::to_string_pretty(&timing).unwrap()).ok();

    test_dir
}

/// 從 manifest JSON 解析摘要
fn parse_manifest_summary(manifest_json: &str) -> ManifestSummary {
    let value: serde_json::Value = match serde_json::from_str(manifest_json) {
        Ok(v) => v,
        Err(_) => return ManifestSummary::default(),
    };

    let work_mode = value
        .get("work_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let complexity = value.get("complexity").unwrap_or(&serde_json::Value::Null);
    let reasoning_branches = complexity
        .get("reasoning_branches")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u8;
    let domain_diversity = complexity
        .get("domain_diversity")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u8;
    let context_complexity = complexity
        .get("context_complexity")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let dispatch = value.get("dispatch").unwrap_or(&serde_json::Value::Null);
    let estimated_nodes = dispatch
        .get("estimated_nodes")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let domain_tags: Vec<String> = dispatch
        .get("domain_tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    ManifestSummary {
        work_mode,
        reasoning_branches,
        domain_diversity,
        context_complexity,
        estimated_nodes,
        domain_tags,
    }
}

/// 列出所有測試記錄
pub fn list_tests() -> Vec<TestSummary> {
    let tests_dir = evolution_root().join("tests");
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(&tests_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let result_file = entry.path().join("result.json");
                if let Ok(content) = fs::read_to_string(&result_file) {
                    if let Ok(record) = serde_json::from_str::<TestRecord>(&content) {
                        results.push(TestSummary {
                            dir_name: entry.file_name().to_string_lossy().to_string(),
                            timestamp: record.timestamp,
                            task: record.task,
                            result: record.result,
                            duration_ms: record.duration_ms,
                        });
                    }
                }
            }
        }
    }

    // 最新的在前
    results.reverse();
    results
}

/// 測試摘要（用於列表展示）
#[derive(Debug, serde::Serialize)]
pub struct TestSummary {
    pub dir_name: String,
    pub timestamp: String,
    pub task: String,
    pub result: String,
    pub duration_ms: u64,
}

/// 輸出所有測試記錄（供 AI 閱讀）
pub fn all_tests_json() -> String {
    let tests = list_tests();
    serde_json::to_string_pretty(&tests).unwrap_or_else(|_| "[]".to_string())
}

/// 取得 Evolution OS 根目錄
pub fn evolution_root() -> PathBuf {
    match std::env::consts::OS {
        "windows" => {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".evolution")
        }
        _ => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".evolution")
        }
    }
}

// Default impl
impl Default for ManifestSummary {
    fn default() -> Self {
        ManifestSummary {
            work_mode: "unknown".to_string(),
            reasoning_branches: 0,
            domain_diversity: 0,
            context_complexity: 0.0,
            estimated_nodes: 0,
            domain_tags: vec![],
        }
    }
}