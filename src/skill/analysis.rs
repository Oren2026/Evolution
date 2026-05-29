//! FileStatsSkill — 分析檔案統計
//!
//! 技能 ID: `filesystem.file_stats`
//!
//! 分析一組檔案的大小、行數、副檔名等。

use crate::skill::{Skill, SkillOutput};
use std::fs;
use std::path::Path;

/// 分析檔案統計的技能
pub struct FileStatsSkill;

impl FileStatsSkill {
    pub fn new() -> Self {
        Self
    }
}

impl Skill for FileStatsSkill {
    fn id(&self) -> &str {
        "filesystem.file_stats"
    }
    fn name(&self) -> &str {
        "File Statistics Analyzer"
    }
    fn description(&self) -> &str {
        "Analyze file statistics: line count, extension, size"
    }
    fn input_format(&self) -> &str {
        r#"{"files": [{"path": "/path/to/file", "name": "file.rs"}, ...]}"#
    }
    fn output_format(&self) -> &str {
        r#"{"stats": [{"path": "...", "lines": 100, "extension": "rs", "bytes": 2048}, ...]}"#
    }
    fn triggers(&self) -> Vec<&str> {
        vec!["stats", "analyze", "lines", "count", "extension"]
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["filesystem.read_dir"]
    }
    fn execute(&self, input: &str) -> SkillOutput {
        // 解析輸入
        let files = match parse_file_list(input) {
            Ok(f) => f,
            Err(e) => return SkillOutput::err(&e),
        };

        let mut stats = Vec::new();
        for file in files {
            let path = Path::new(&file.path);

            // 讀取檔案統計
            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let lines = if metadata.is_file() {
                fs::read_to_string(path)
                    .map(|content| content.lines().count() as u32)
                    .unwrap_or(0)
            } else {
                0
            };

            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();

            stats.push(serde_json::json!({
                "path": file.path,
                "name": file.name,
                "lines": lines,
                "extension": extension,
                "bytes": metadata.len(),
                "is_dir": metadata.is_dir(),
            }));
        }

        let result = serde_json::json!({
            "stats": stats,
            "total_files": stats.len(),
        });

        SkillOutput::ok(&result.to_string())
    }
}

// ===== 工具函式 =====

struct FileEntry {
    path: String,
    name: String,
}

fn parse_file_list(input: &str) -> Result<Vec<FileEntry>, String> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        // 支援 {"files": [...]} 格式
        if let Some(files_arr) = json.get("files").and_then(|f| f.as_array()) {
            let entries = files_arr
                .iter()
                .filter_map(|f| {
                    let path = f.get("path")?.as_str()?.to_string();
                    let name = f.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                    Some(FileEntry { path, name })
                })
                .collect();
            return Ok(entries);
        }
    }

    // Fallback：直接作為路徑處理
    Ok(vec![FileEntry {
        path: input.trim().to_string(),
        name: Path::new(input.trim())
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
    }])
}

/// 計算目錄下所有檔案的統計（遞迴）
pub fn count_lines_recursive(path: &std::path::Path) -> u32 {
    let mut total = 0;

    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // 忽略 target、.git 等目錄
                    if let Some(name) = entry_path.file_name() {
                        let name = name.to_string_lossy();
                        if name.starts_with('.') || name == "target" || name == "node_modules" {
                            continue;
                        }
                    }
                    total += count_lines_recursive(&entry_path);
                } else if entry_path.is_file() {
                    total += fs::read_to_string(&entry_path)
                        .map(|c| c.lines().count() as u32)
                        .unwrap_or(0);
                }
            }
        }
    } else if path.is_file() {
        total = fs::read_to_string(path)
            .map(|c| c.lines().count() as u32)
            .unwrap_or(0);
    }

    total
}

// ===== ProjectAnalyzerSkill =====

/// 專案分析器技能 — 根據目錄結構和檔案統計產生摘要
pub struct ProjectAnalyzerSkill;

impl ProjectAnalyzerSkill {
    pub fn new() -> Self {
        Self
    }
}

impl Skill for ProjectAnalyzerSkill {
    fn id(&self) -> &str {
        "analysis.project_analyzer"
    }
    fn name(&self) -> &str {
        "Project Analyzer"
    }
    fn description(&self) -> &str {
        "Summarize project structure: modules, tests, line counts"
    }
    fn input_format(&self) -> &str {
        r#"{"dirs": [...], "files": [...], "stats": {...}})"#
    }
    fn output_format(&self) -> &str {
        r#"{"summary": "text", "modules": [...], "total_lines": N}"#
    }
    fn triggers(&self) -> Vec<&str> {
        vec!["project", "analyze", "summary", "structure", "modules"]
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["filesystem.read_dir", "filesystem.file_stats"]
    }
    fn execute(&self, input: &str) -> SkillOutput {
        // 解析輸入
        let json = match serde_json::from_str::<serde_json::Value>(input) {
            Ok(j) => j,
            Err(e) => return SkillOutput::err(&format!("invalid input JSON: {}", e)),
        };

        // 嘗試從 input 本身解析 stats（如果是直接呼叫）
        let total_lines = if let Some(stats) = json.get("stats").and_then(|s| s.get("stats")) {
            stats
                .as_array()
                .map(|arr| arr.iter().filter_map(|s| s.get("lines").and_then(|l| l.as_u64())).sum::<u64>() as u32)
                .unwrap_or(0)
        } else {
            0
        };

        // 解析目錄結構
        let dirs = json
            .get("dirs")
            .and_then(|d| d.as_array())
            .map(|arr| arr.len() as u32)
            .unwrap_or(0);

        let files = json
            .get("files")
            .and_then(|f| f.as_array())
            .map(|arr| arr.len() as u32)
            .unwrap_or(0);

        // 解析模組（src/ 下的目錄）
        let modules: Vec<String> = json
            .get("dirs")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.get("name").and_then(|n| n.as_str()))
                    .filter(|name| !name.starts_with('.') && *name != "target")
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // 產生摘要
        let summary = format!(
            "Project has {} modules, {} files, {} lines of code",
            modules.len(),
            files,
            total_lines
        );

        let result = serde_json::json!({
            "summary": summary,
            "modules": modules,
            "total_dirs": dirs,
            "total_files": files,
            "total_lines": total_lines,
            "has_tests": true,
        });

        SkillOutput::ok(&result.to_string())
    }
}

// ===== 測試 =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, DirBuilder};
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("evolution_os_stats_test");
        let _ = DirBuilder::new().recursive(true).create(&dir);
        let _ = fs::write(dir.join("mod.rs"), "fn main() {}\nfn foo() {}");
        let _ = fs::write(dir.join("lib.rs"), "pub fn bar() {}\n");
        dir
    }

    #[test]
    fn test_file_stats_basic() {
        let skill = FileStatsSkill::new();
        let dir = test_dir();

        let input = serde_json::json!({
            "files": [
                {"path": dir.join("mod.rs").to_string_lossy(), "name": "mod.rs"},
                {"path": dir.join("lib.rs").to_string_lossy(), "name": "lib.rs"},
            ]
        })
        .to_string();

        let output = skill.execute(&input);
        assert!(output.success, "expected success, got: {:?}", output.error);
        let data: serde_json::Value = serde_json::from_str(&output.data).unwrap();
        assert_eq!(data["total_files"], 2);
        assert!(data["stats"][0]["lines"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_count_lines_recursive() {
        let dir = test_dir();
        let total = count_lines_recursive(&dir);
        assert!(total > 0, "expected some lines, got {}", total);
    }
}