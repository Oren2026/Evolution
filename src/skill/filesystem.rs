//! FileSystemSkill — 讀取目錄結構
//!
//! 技能 ID: `filesystem.read_dir`

use crate::skill::{Skill, SkillOutput};
use std::fs;

/// 讀取目錄結構的技能
pub struct FileSystemSkill;

impl FileSystemSkill {
    pub fn new() -> Self {
        Self
    }
}

impl Skill for FileSystemSkill {
    fn id(&self) -> &str {
        "filesystem.read_dir"
    }
    fn name(&self) -> &str {
        "File System Reader"
    }
    fn description(&self) -> &str {
        "Read directory structure and list files"
    }
    fn input_format(&self) -> &str {
        r#"{"path": "/Users/example/project"}"#
    }
    fn output_format(&self) -> &str {
        r#"{"files": [{"name": "src", "is_dir": true}, ...], "dirs": [...]}"#
    }
    fn triggers(&self) -> Vec<&str> {
        vec!["directory", "read", "list", "files", "folder"]
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    fn execute(&self, input: &str) -> SkillOutput {
        // 解析輸入
        let path = match extract_path(input) {
            Ok(p) => p,
            Err(e) => return SkillOutput::err(&e),
        };

        // 讀取目錄
        match fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                let mut dirs = Vec::new();

                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

                    let entry_json = serde_json::json!({
                        "name": name,
                        "is_dir": is_dir,
                    });

                    if is_dir {
                        dirs.push(entry_json);
                    } else {
                        files.push(entry_json);
                    }
                }

                let result = serde_json::json!({
                    "path": path,
                    "files": files,
                    "dirs": dirs,
                });

                SkillOutput::ok(&result.to_string())
            }
            Err(e) => SkillOutput::err(&format!("failed to read directory '{}': {}", path, e)),
        }
    }
}

// ===== 工具函式 =====

fn extract_path(input: &str) -> Result<String, String> {
    // 嘗試從 JSON 解析
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
        if let Some(path) = json.get("path").and_then(|p| p.as_str()) {
            return Ok(expand_tilde(path));
        }
    }

    // Fallback：直接用 input 作為路徑
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("empty path".to_string());
    }
    Ok(expand_tilde(trimmed))
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replace("~", &home);
        }
    }
    path.to_string()
}

// ===== 測試 =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, DirBuilder};
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("evolution_os_fs_test");
        let _ = DirBuilder::new().recursive(true).create(&dir);
        // 建立測試檔案
        let _ = fs::write(dir.join("file1.txt"), "content");
        let _ = fs::write(dir.join("file2.rs"), "fn main() {}");
        let _ = fs::create_dir(dir.join("subdir"));
        dir
    }

    #[test]
    fn test_read_dir_basic() {
        let skill = FileSystemSkill::new();
        let dir = test_dir();

        let input = serde_json::json!({"path": dir.to_string_lossy()}).to_string();
        let output = skill.execute(&input);

        assert!(output.success, "expected success, got: {:?}", output.error);
        let data: serde_json::Value = serde_json::from_str(&output.data).unwrap();
        assert!(data["files"].is_array());
        assert!(data["dirs"].is_array());
        assert_eq!(data["dirs"].as_array().unwrap().len(), 1); // subdir
    }

    #[test]
    fn test_path_expansion() {
        let skill = FileSystemSkill::new();
        // 使用 temp 目錄當作 ~ 的替代
        let input = serde_json::json!({"path": "/tmp"}).to_string();
        let output = skill.execute(&input);
        assert!(output.success);
    }

    #[test]
    fn test_nonexistent_path() {
        let skill = FileSystemSkill::new();
        let input = serde_json::json!({"path": "/nonexistent/path/12345"}).to_string();
        let output = skill.execute(&input);
        assert!(!output.success, "should fail for nonexistent path");
    }
}