//! 系統環境自動安裝器

use std::process::Command;

/// 安裝進度回呼
pub type InstallCallback = fn(stage: &str, message: &str);

/// 簡單不輸出的 callback
fn noop(_: &str, _: &str) {}

/// 安裝器本體
pub struct Installer {
    callback: InstallCallback,
}

impl Installer {
    pub fn new() -> Self {
        Self { callback: noop }
    }

    pub fn with_callback(mut self, cb: InstallCallback) -> Self {
        self.callback = cb;
        self
    }

    fn stage(&self, msg: &str) {
        println!("{}", msg);
    }

    /// 嘗試安裝所有缺少的元件
    /// 返回 true 全部成功，false 有失敗
    pub fn install_missing(&self) -> bool {
        let mut all_ok = true;

        // 1. Rust
        if !Self::is_rust_installed() {
            self.stage("📦 安裝 Rust...");
            if !self.install_rust() {
                all_ok = false;
            }
        } else {
            self.stage("✅ Rust/Cargo 已就緒");
        }

        // 2. Ollama
        if !Self::is_ollama_installed() {
            self.stage("📦 安裝 Ollama...");
            if !self.install_ollama() {
                all_ok = false;
            }
        } else {
            self.stage("✅ Ollama 已就緒");
        }

        // 3. llama3 模型
        if !Self::is_llama3_installed() {
            self.stage("📦 拉取 llama3 模型（約 4-5GB，首次可能需要幾分鐘）...");
            if !self.pull_llama3() {
                all_ok = false;
            }
        } else {
            self.stage("✅ llama3 模型已就緒");
        }

        all_ok
    }

    pub(crate) fn is_rust_installed() -> bool {
        Command::new("cargo")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_rust(&self) -> bool {
        self.stage("   執行: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y");
        let output = Command::new("sh")
            .args(["-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"])
            .output();

        match output {
            Ok(o) => {
                if o.status.success() {
                    self.stage("   Rust 安裝完成（需重新載入 shell 或 source ~/.cargo/env）");
                    // 嘗試 source 後驗證
                    let check = Command::new("sh")
                        .args(["-c", "source ~/.cargo/env 2>/dev/null; cargo --version"])
                        .output();
                    check.map(|c| c.status.success()).unwrap_or(false)
                } else {
                    eprintln!("   ❌ Rust 安裝失敗: {}", String::from_utf8_lossy(&o.stderr));
                    false
                }
            }
            Err(e) => {
                eprintln!("   ❌ 無法執行安裝腳本: {}", e);
                false
            }
        }
    }

    pub(crate) fn is_ollama_installed() -> bool {
        // 先嘗試 which
        if Command::new("which")
            .arg("ollama")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return true;
        }
        // 或檢查常見路徑
        if std::fs::metadata("/usr/local/bin/ollama").is_ok() {
            return true;
        }
        if std::fs::metadata("/opt/homebrew/bin/ollama").is_ok() {
            return true;
        }
        // 或檢查本地服務
        Command::new("curl")
            .args(["-s", "-m", "2", "http://localhost:11434/api/tags"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_ollama(&self) -> bool {
        // macOS 用 brew
        #[cfg(target_os = "macos")]
        {
            self.stage("   執行: brew install ollama");
            let output = Command::new("brew").args(["install", "ollama"]).output();
            match output {
                Ok(o) if o.status.success() => {
                    self.stage("   ✅ Ollama 安裝完成，正在啟動服務...");
                    // 啟動 ollama service
                    let _ = Command::new("brew").args(["services", "start", "ollama"]).output();
                    // 等一下讓服務啟動
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    true
                }
                Ok(o) => {
                    eprintln!("   ❌ brew 安裝失敗: {}", String::from_utf8_lossy(&o.stderr));
                    false
                }
                Err(e) => {
                    eprintln!("   ❌ 找不到 brew: {}", e);
                    false
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Linux / 其他平台
            self.stage("   請手動安裝 Ollama: https://github.com/ollama/ollama");
            false
        }
    }

    pub(crate) fn is_llama3_installed() -> bool {
        let ollama_bin = if std::fs::metadata("/usr/local/bin/ollama").is_ok() {
            Some("/usr/local/bin/ollama")
        } else if std::fs::metadata("/opt/homebrew/bin/ollama").is_ok() {
            Some("/opt/homebrew/bin/ollama")
        } else if Command::new("which").arg("ollama").output().map(|o| o.status.success()).unwrap_or(false) {
            Some("ollama")
        } else {
            None
        };

        match ollama_bin {
            Some(path) => Command::new(path)
                .arg("list")
                .output()
                .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("llama3"))
                .unwrap_or(false),
            None => false,
        }
    }

    fn pull_llama3(&self) -> bool {
        let ollama_bin = Self::find_ollama_bin();
        self.stage("   執行: ollama pull llama3");
        self.stage("   下載進度（首次需 4-5 分鐘，請耐心等待）...\n");

        // streaming pull，这样用户能看到下載進度
        let mut child = match Command::new(ollama_bin)
            .args(["pull", "llama3"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("   ❌ 無法啟動 ollama pull: {}", e);
                return false;
            }
        };

        // 讀取 stdout + stderr 並即時列印
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // 用一個執行緒讀 stdout，另一個讀 stderr
        let stdout_handle = stdout.map(|mut io| {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(io);
                for line in reader.lines().map_while(Result::ok) {
                    // 顯示进度行
                    if !line.trim().is_empty() {
                        println!("   {}", line);
                    }
                }
            })
        });

        let stderr_handle = stderr.map(|mut io| {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(io);
                for line in reader.lines().map_while(Result::ok) {
                    if !line.trim().is_empty() {
                        eprintln!("   [stderr] {}", line);
                    }
                }
            })
        });

        // 等待 child 結束
        let status = child.wait();

        // 等待 stdout/stderr 執行緒结束
        if let Some(h) = stdout_handle {
            let _ = h.join();
        }
        if let Some(h) = stderr_handle {
            let _ = h.join();
        }

        match status {
            Ok(s) if s.success() => {
                self.stage("   ✅ llama3 模型拉取完成");
                true
            }
            Ok(s) => {
                // pull 成功但 status 非0（可能是已存在）
                if Self::is_llama3_installed() {
                    self.stage("   ✅ llama3 模型已就緒");
                    true
                } else {
                    eprintln!("   ❌ llama3 pull 返回非零狀態: {}", s);
                    false
                }
            }
            Err(e) => {
                eprintln!("   ❌ ollama pull 執行失敗: {}", e);
                false
            }
        }
    }

    fn find_ollama_bin() -> &'static str {
        if std::fs::metadata("/usr/local/bin/ollama").is_ok() {
            "/usr/local/bin/ollama"
        } else if std::fs::metadata("/opt/homebrew/bin/ollama").is_ok() {
            "/opt/homebrew/bin/ollama"
        } else {
            "ollama"
        }
    }

    /// 快速預檢（不安装，只檢查）
    pub fn quick_check() -> Vec<(String, bool)> {
        vec![
            ("Rust/Cargo".to_string(), Self::is_rust_installed()),
            ("Ollama".to_string(), Self::is_ollama_installed()),
            ("llama3 model".to_string(), Self::is_llama3_installed()),
        ]
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::new()
    }
}
