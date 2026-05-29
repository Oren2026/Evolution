//! 系統環境檢查器 — 檢測必要軟體是否存在

use std::process::Command;

/// 檢查結果
#[derive(Debug, Clone)]
pub enum CheckStatus {
    Installed(String),     // 已安裝，版本字串
    Missing,                // 找不到
    CommandFailed(String),  // 執行失敗（通常是找不到指令）
}

impl CheckStatus {
    pub fn is_installed(&self) -> bool {
        matches!(self, CheckStatus::Installed(_))
    }
}

/// 單一軟體的檢查結果
#[derive(Debug, Clone)]
pub struct CheckItem {
    pub name: String,
    pub status: CheckStatus,
}

impl CheckItem {
    pub fn new(name: &str, status: CheckStatus) -> Self {
        Self {
            name: name.to_string(),
            status,
        }
    }
}

/// 完整檢查報告
#[derive(Debug, Clone)]
pub struct SystemReport {
    pub items: Vec<CheckItem>,
}

impl SystemReport {
    pub fn rust() -> Self {
        let output = Command::new("cargo").arg("--version").output();
        let status = match output {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                CheckStatus::Installed(version)
            }
            _ => CheckStatus::Missing,
        };
        Self {
            items: vec![CheckItem::new("Rust/Cargo", status)],
        }
    }

    pub fn ollama() -> Self {
        // 先檢查 which，成功就用
        let output = Command::new("which").arg("ollama").output();
        let status = match output {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                CheckStatus::Installed(version)
            }
            _ => {
                // 嘗試常見安裝路徑
                let paths = ["/usr/local/bin/ollama", "/opt/homebrew/bin/ollama"];
                let found = paths.iter().find(|p| std::fs::metadata(p).is_ok());
                match found {
                    Some(p) => CheckStatus::Installed(p.to_string()),
                    None => {
                        // 最後 fallback: 試 curl 本地服務
                        let curl = Command::new("curl")
                            .args(["-s", "-m", "2", "http://localhost:11434/api/tags"])
                            .output();
                        match curl {
                            Ok(out) if out.status.success() => {
                                CheckStatus::Installed("running (localhost:11434)".to_string())
                            }
                            _ => CheckStatus::Missing,
                        }
                    }
                }
            }
        };
        Self {
            items: vec![CheckItem::new("Ollama", status)],
        }
    }

    pub fn llama3() -> Self {
        // try /usr/local/bin/ollama directly since it's not in PATH
        let ollama_bin = if std::fs::metadata("/usr/local/bin/ollama").is_ok() {
            Some("/usr/local/bin/ollama")
        } else if std::fs::metadata("/opt/homebrew/bin/ollama").is_ok() {
            Some("/opt/homebrew/bin/ollama")
        } else {
            None
        };

        let status = match ollama_bin {
            Some(path) => {
                let output = Command::new(path).arg("list").output();
                match output {
                    Ok(out) if out.status.success() => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if stdout.contains("llama3") {
                            CheckStatus::Installed("llama3".to_string())
                        } else {
                            CheckStatus::Missing
                        }
                    }
                    _ => CheckStatus::Missing,
                }
            }
            None => CheckStatus::Missing,
        };
        Self {
            items: vec![CheckItem::new("llama3 model", status)],
        }
    }

    pub fn all() -> Vec<SystemReport> {
        vec![Self::rust(), Self::ollama(), Self::llama3()]
    }

    pub fn print_summary(&self) {
        for item in &self.items {
            match &item.status {
                CheckStatus::Installed(v) => {
                    println!("  ✅ {} — {}", item.name, v);
                }
                CheckStatus::Missing => {
                    println!("  ❌ {} — 未安裝", item.name);
                }
                CheckStatus::CommandFailed(e) => {
                    println!("  ⚠️  {} — 檢查失敗: {}", item.name, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_check() {
        let report = SystemReport::rust();
        assert_eq!(report.items.len(), 1);
        assert!(report.items[0].status.is_installed());
    }

    #[test]
    fn test_ollama_check() {
        let report = SystemReport::ollama();
        assert_eq!(report.items.len(), 1);
    }

    #[test]
    fn test_llama3_check() {
        let report = SystemReport::llama3();
        assert_eq!(report.items.len(), 1);
    }
}
